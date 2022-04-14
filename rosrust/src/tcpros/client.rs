use super::error::{ErrorKind, Result, ResultExt};
use super::header::{decode, encode};
use super::{ServicePair, ServiceResult};
use crate::api::Master;
use crate::rosmsg::RosMsg;
use byteorder::{LittleEndian, ReadBytesExt};
use error_chain::bail;
use log::error;
use socket2::Socket;
use std::collections::HashMap;
use std::io;
use std::io::{Read, Write};
use std::net::TcpStream;
use std::sync::Arc;
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

#[derive(Clone)]
pub struct Client<T: ServicePair> {
    master: std::sync::Arc<Master>,
    info: std::sync::Arc<ClientInfo>,
    phantom: std::marker::PhantomData<T>,
}

fn connect_to_tcp_attempt(master: &Master, service: &str) -> Result<TcpStream> {
    let uri = master
        .lookup_service(&service)
        .chain_err(|| ErrorKind::ServiceConnectionFail(service.into()))?;
    let trimmed_uri = uri.trim_start_matches("rosrpc://");
    let stream = TcpStream::connect(trimmed_uri)?;
    let socket: Socket = stream.into();
    socket.set_linger(None)?;
    let stream: TcpStream = socket.into();
    Ok(stream)
}

fn connect_to_tcp_with_multiple_attempts(
    master: &Master,
    name: &str,
    attempts: usize,
) -> Result<TcpStream> {
    let mut err = io::Error::new(
        io::ErrorKind::Other,
        "Tried to connect via TCP with 0 connection attempts",
    )
    .into();
    let mut repeat_delay_ms = 1;
    for _ in 0..attempts {
        let stream_result = connect_to_tcp_attempt(master, name);
        match stream_result {
            Ok(stream) => {
                return Ok(stream);
            }
            Err(error) => err = error,
        }
        std::thread::sleep(std::time::Duration::from_millis(repeat_delay_ms));
        repeat_delay_ms *= 2;
    }
    Err(err)
}

impl<T: ServicePair> Client<T> {
    pub fn new(master: Arc<Master>, caller_id: &str, service: &str) -> Client<T> {
        Client {
            master,
            info: std::sync::Arc::new(ClientInfo {
                caller_id: String::from(caller_id),
                service: String::from(service),
            }),
            phantom: std::marker::PhantomData,
        }
    }

    pub fn req(&self, args: &T::Request) -> Result<ServiceResult<T::Response>> {
        Self::request_body(
            args,
            Arc::clone(&self.master),
            &self.info.caller_id,
            &self.info.service,
        )
    }

    pub fn req_async(&self, args: T::Request) -> ClientResponse<T::Response> {
        let info = Arc::clone(&self.info);
        let master = Arc::clone(&self.master);
        ClientResponse {
            handle: thread::spawn(move || {
                Self::request_body(&args, master, &info.caller_id, &info.service)
            }),
        }
    }

    fn request_body(
        args: &T::Request,
        master: Arc<Master>,
        caller_id: &str,
        service: &str,
    ) -> Result<ServiceResult<T::Response>> {
        let mut stream = connect_to_tcp_with_multiple_attempts(master.as_ref(), service, 15)
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

fn read_response<T, U>(mut stream: &mut U) -> Result<()>
where
    T: ServicePair,
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
    read_response::<T, U>(stream)
}
