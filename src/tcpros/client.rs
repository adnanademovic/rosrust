use rustc_serialize::Decodable;
use std::net::TcpStream;
use std::thread;
use std::collections::HashMap;
use std;
use serde_rosmsg::to_writer;
use super::error::{ErrorKind, Result, ResultExt};
use super::header::{encode, decode};
use super::{ServicePair, ServiceResult};
use super::decoder::DecoderSource;

pub struct ClientResponse<T> {
    handle: thread::JoinHandle<Result<ServiceResult<T>>>,
}

impl<T> ClientResponse<T> {
    pub fn read(self) -> Result<ServiceResult<T>> {
        self.handle.join().unwrap_or(Err(ErrorKind::ServiceResponseUnknown.into()))
    }
}

impl<T: Send + 'static> ClientResponse<T> {
    pub fn callback<F>(self, callback: F)
        where F: FnOnce(Result<ServiceResult<T>>) + Send + 'static
    {
        thread::spawn(move || callback(self.read()));
    }
}

pub struct Client<T: ServicePair> {
    caller_id: String,
    uri: String,
    service: String,
    phantom: std::marker::PhantomData<T>,
}

impl<T: ServicePair> Client<T> {
    pub fn new(caller_id: &str, uri: &str, service: &str) -> Client<T> {
        Client {
            caller_id: String::from(caller_id),
            uri: String::from(uri),
            service: String::from(service),
            phantom: std::marker::PhantomData,
        }
    }

    pub fn req(&self, args: &T::Request) -> Result<ServiceResult<T::Response>> {
        Self::request_body(args, &self.uri, &self.caller_id, &self.service)
    }

    pub fn req_async(&self, args: T::Request) -> ClientResponse<T::Response> {
        let uri = self.uri.clone();
        let caller_id = self.caller_id.clone();
        let service = self.service.clone();
        ClientResponse {
            handle: thread::spawn(move || Self::request_body(&args, &uri, &caller_id, &service)),
        }
    }

    fn request_body(args: &T::Request,
                    uri: &str,
                    caller_id: &str,
                    service: &str)
                    -> Result<ServiceResult<T::Response>> {
        let connection = TcpStream::connect(uri.trim_left_matches("rosrpc://"));
        let mut stream =
            connection.chain_err(|| ErrorKind::ServiceConnectionFail(service.into(), uri.into()))?;
        exchange_headers::<T, _>(&mut stream, caller_id, service)?;

        to_writer(&mut stream, &args)?;

        let mut decoder = DecoderSource::new(&mut stream);
        let success = decoder.pop_verification_byte()
            .chain_err(|| ErrorKind::ServiceResponseInterruption)?;
        let mut decoder = decoder.pop_decoder()
            .chain_err(|| ErrorKind::ServiceResponseInterruption)?;
        Ok(if success {
            Ok(T::Response::decode(&mut decoder)?)
        } else {
            Err(String::decode(&mut decoder)?)
        })
    }
}

fn write_request<T, U>(mut stream: &mut U, caller_id: &str, service: &str) -> Result<()>
    where T: ServicePair,
          U: std::io::Write
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
    where T: ServicePair,
          U: std::io::Read
{
    let fields = decode(&mut stream)?;
    if fields.get("callerid").is_none() {
        bail!(ErrorKind::HeaderMissingField("callerid".into()));
    }
    Ok(())
}

fn exchange_headers<T, U>(mut stream: &mut U, caller_id: &str, service: &str) -> Result<()>
    where T: ServicePair,
          U: std::io::Write + std::io::Read
{
    write_request::<T, U>(stream, caller_id, service)?;
    read_response::<T, U>(stream)
}
