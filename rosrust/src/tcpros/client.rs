use super::error::{ErrorKind, Result, ResultExt};
use super::header::{decode, encode};
use super::{ServicePair, ServiceResult};
use crate::api::Master;
use crate::rosmsg::RosMsg;
use crate::util::FAILED_TO_LOCK;
use byteorder::{LittleEndian, ReadBytesExt};
use error_chain::bail;
use log::error;
use socket2::Socket;
use std::collections::HashMap;
use std::io;
use std::io::{Read, Write};
use std::net::{TcpStream, ToSocketAddrs};
use std::sync::{Arc, Mutex};
use std::thread;

pub struct ClientResponse<T> {
    handle: thread::JoinHandle<Result<ServiceResult<T>>>,
}

impl<T> ClientResponse<T> {
    pub fn read(self) -> Result<ServiceResult<T>> {
        self.handle
            .join()
            .unwrap_or_else(|_| Err(ErrorKind::ServiceResponseUnknown.into()))
    }
}

impl<T: Send + 'static> ClientResponse<T> {
    pub fn callback<F>(self, callback: F)
    where
        F: FnOnce(Result<ServiceResult<T>>) + Send + 'static,
    {
        thread::spawn(move || callback(self.read()));
    }
}

struct ClientInfo {
    caller_id: String,
    service: String,
}

struct UriCache {
    master: std::sync::Arc<Master>,
    data: Mutex<Option<String>>,
    service: String,
}

impl UriCache {
    fn get(&self) -> Result<String> {
        if let Some(uri) = Option::<String>::clone(&self.data.lock().expect(FAILED_TO_LOCK)) {
            return Ok(uri);
        }
        let new_uri = self
            .master
            .lookup_service(&self.service)
            .chain_err(|| ErrorKind::ServiceConnectionFail(self.service.clone()))?;
        *self.data.lock().expect(FAILED_TO_LOCK) = Some(new_uri.clone());
        Ok(new_uri)
    }

    fn clear(&self) {
        *self.data.lock().expect(FAILED_TO_LOCK) = None;
    }
}

#[derive(Clone)]
pub struct Client<T: ServicePair> {
    info: std::sync::Arc<ClientInfo>,
    uri_cache: std::sync::Arc<UriCache>,
    phantom: std::marker::PhantomData<T>,
}

fn connect_to_tcp_attempt(
    uri_cache: &UriCache,
    timeout: Option<std::time::Duration>,
) -> Result<TcpStream> {
    let uri = uri_cache.get()?;
    let trimmed_uri = uri.trim_start_matches("rosrpc://");
    let stream = match timeout {
        Some(timeout) => {
            let invalid_addr_error = || {
                ErrorKind::Io(io::Error::new(
                    io::ErrorKind::Other,
                    "Invalid socket address",
                ))
            };
            let socket_addr = trimmed_uri
                .to_socket_addrs()
                .chain_err(invalid_addr_error)?
                .next()
                .ok_or_else(invalid_addr_error)?;
            TcpStream::connect_timeout(&socket_addr, timeout)?
        }
        None => TcpStream::connect(trimmed_uri)?,
    };
    let socket: Socket = stream.into();
    if let Some(timeout) = timeout {
        // In case defaults are not None, only apply if a timeout is passed
        socket.set_read_timeout(Some(timeout))?;
        socket.set_write_timeout(Some(timeout))?;
    }
    socket.set_linger(None)?;
    let stream: TcpStream = socket.into();
    Ok(stream)
}

fn connect_to_tcp_with_multiple_attempts(
    uri_cache: &UriCache,
    attempts: usize,
) -> Result<TcpStream> {
    let mut err = io::Error::new(
        io::ErrorKind::Other,
        "Tried to connect via TCP with 0 connection attempts",
    )
    .into();
    let mut repeat_delay_ms = 1;
    for _ in 0..attempts {
        let stream_result = connect_to_tcp_attempt(uri_cache, None);
        match stream_result {
            Ok(stream) => {
                return Ok(stream);
            }
            Err(error) => err = error,
        }
        uri_cache.clear();
        std::thread::sleep(std::time::Duration::from_millis(repeat_delay_ms));
        repeat_delay_ms *= 2;
    }
    Err(err)
}

impl<T: ServicePair> Client<T> {
    pub fn new(master: Arc<Master>, caller_id: &str, service: &str) -> Client<T> {
        Client {
            info: std::sync::Arc::new(ClientInfo {
                caller_id: String::from(caller_id),
                service: String::from(service),
            }),
            uri_cache: std::sync::Arc::new(UriCache {
                master,
                data: Mutex::new(None),
                service: String::from(service),
            }),
            phantom: std::marker::PhantomData,
        }
    }

    fn probe_inner(&self, timeout: std::time::Duration) -> Result<()> {
        let mut stream = connect_to_tcp_attempt(&self.uri_cache, Some(timeout))?;
        exchange_probe_headers(&mut stream, &self.info.caller_id, &self.info.service)?;
        Ok(())
    }

    pub fn probe(&self, timeout: std::time::Duration) -> Result<()> {
        let probe_result = self.probe_inner(timeout);
        if probe_result.is_err() {
            self.uri_cache.clear();
        }
        probe_result
    }

    pub fn req(&self, args: &T::Request) -> Result<ServiceResult<T::Response>> {
        Self::request_body(
            args,
            &self.uri_cache,
            &self.info.caller_id,
            &self.info.service,
        )
    }

    pub fn req_async(&self, args: T::Request) -> ClientResponse<T::Response> {
        let info = Arc::clone(&self.info);
        let uri_cache = Arc::clone(&self.uri_cache);
        ClientResponse {
            handle: thread::spawn(move || {
                Self::request_body(&args, &uri_cache, &info.caller_id, &info.service)
            }),
        }
    }

    fn request_body(
        args: &T::Request,
        uri_cache: &UriCache,
        caller_id: &str,
        service: &str,
    ) -> Result<ServiceResult<T::Response>> {
        let mut stream = connect_to_tcp_with_multiple_attempts(uri_cache, 15)
            .chain_err(|| ErrorKind::ServiceConnectionFail(service.into()))?;

        // Service request starts by exchanging connection headers
        exchange_headers::<T, _>(&mut stream, caller_id, service)?;

        let mut writer = io::Cursor::new(Vec::with_capacity(128));
        // skip the first 4 bytes that will contain the message length
        writer.set_position(4);

        args.encode(&mut writer)?;

        // write the message length to the start of the header
        let message_length = (writer.position() - 4) as u32;
        writer.set_position(0);
        message_length.encode(&mut writer)?;

        // Send request to service
        stream.write_all(&writer.into_inner())?;

        // Service responds with a boolean byte, signalling success
        let success = read_verification_byte(&mut stream)
            .chain_err(|| ErrorKind::ServiceResponseInterruption)?;
        Ok(if success {
            // Decode response as response type upon success

            // TODO: validate response length
            let _length = stream.read_u32::<LittleEndian>();

            let data = RosMsg::decode(&mut stream)?;

            let mut dump = vec![];
            if let Err(err) = stream.read_to_end(&mut dump) {
                error!("Failed to read from TCP stream: {:?}", err)
            }

            Ok(data)
        } else {
            // Decode response as string upon failure
            let data = RosMsg::decode(&mut stream)?;

            let mut dump = vec![];
            if let Err(err) = stream.read_to_end(&mut dump) {
                error!("Failed to read from TCP stream: {:?}", err)
            }

            Err(data)
        })
    }
}

#[inline]
fn read_verification_byte<R: std::io::Read>(reader: &mut R) -> std::io::Result<bool> {
    reader.read_u8().map(|v| v != 0)
}

fn write_request<T, U>(mut stream: &mut U, caller_id: &str, service: &str) -> Result<()>
where
    T: ServicePair,
    U: std::io::Write,
{
    let mut fields = HashMap::<String, String>::new();
    fields.insert(String::from("callerid"), String::from(caller_id));
    fields.insert(String::from("service"), String::from(service));
    fields.insert(String::from("md5sum"), T::md5sum());
    fields.insert(String::from("type"), T::msg_type());
    encode(&mut stream, &fields)?;
    Ok(())
}

fn write_probe_request<U>(mut stream: &mut U, caller_id: &str, service: &str) -> Result<()>
where
    U: std::io::Write,
{
    let mut fields = HashMap::<String, String>::new();
    fields.insert(String::from("probe"), String::from("1"));
    fields.insert(String::from("callerid"), String::from(caller_id));
    fields.insert(String::from("service"), String::from(service));
    fields.insert(String::from("md5sum"), String::from("*"));
    encode(&mut stream, &fields)?;
    Ok(())
}

fn read_response<U>(mut stream: &mut U) -> Result<()>
where
    U: std::io::Read,
{
    let fields = decode(&mut stream)?;
    if fields.get("callerid").is_none() {
        bail!(ErrorKind::HeaderMissingField("callerid".into()));
    }
    Ok(())
}

fn exchange_headers<T, U>(stream: &mut U, caller_id: &str, service: &str) -> Result<()>
where
    T: ServicePair,
    U: std::io::Write + std::io::Read,
{
    write_request::<T, U>(stream, caller_id, service)?;
    read_response::<U>(stream)
}

fn exchange_probe_headers<U>(stream: &mut U, caller_id: &str, service: &str) -> Result<()>
where
    U: std::io::Write + std::io::Read,
{
    write_probe_request::<U>(stream, caller_id, service)?;
    read_response::<U>(stream)
}
