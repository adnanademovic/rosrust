use rustc_serialize::{Encodable, Decodable};
use std::net::TcpStream;
use std::thread;
use std::collections::HashMap;
use std;
use super::error::{Error, ErrorKind, ResultExt};
use super::header::{encode, decode};
use super::{ServicePair, ServiceResult};
use super::decoder::DecoderSource;
use super::encoder::Encoder;

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
            uri: String::from(uri.trim_left_matches("rosrpc://")),
            service: String::from(service),
            phantom: std::marker::PhantomData,
        }
    }

    pub fn req(&self, args: &T::Request) -> Result<ServiceResult<T::Response>, Error> {
        Self::request_body(args, &self.uri, &self.caller_id, &self.service)
    }

    pub fn req_callback<F>(&self, args: T::Request, callback: F)
        where F: FnOnce(Result<ServiceResult<T::Response>, Error>) + Send + 'static
    {
        let uri = self.uri.clone();
        let caller_id = self.caller_id.clone();
        let service = self.service.clone();
        thread::spawn(move || callback(Self::request_body(&args, &uri, &caller_id, &service)));
    }

    fn request_body(args: &T::Request,
                    uri: &str,
                    caller_id: &str,
                    service: &str)
                    -> Result<ServiceResult<T::Response>, Error> {
        let mut stream = TcpStream::connect(uri)?;
        exchange_headers::<T, _>(&mut stream, caller_id, service)?;

        let mut encoder = Encoder::new();
        args.encode(&mut encoder)?;
        encoder.write_to(&mut stream)?;

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

fn write_request<T, U>(mut stream: &mut U, caller_id: &str, service: &str) -> Result<(), Error>
    where T: ServicePair,
          U: std::io::Write
{
    let mut fields = HashMap::<String, String>::new();
    fields.insert(String::from("callerid"), String::from(caller_id));
    fields.insert(String::from("service"), String::from(service));
    fields.insert(String::from("md5sum"), T::md5sum());
    fields.insert(String::from("type"), T::msg_type());
    encode(fields)?.write_to(&mut stream)?;
    Ok(())
}

fn read_response<T, U>(mut stream: &mut U) -> Result<(), Error>
    where T: ServicePair,
          U: std::io::Read
{
    let fields = decode(&mut stream)?;
    if fields.get("callerid").is_none() {
        bail!(ErrorKind::HeaderMissingField("callerid".into()));
    }
    Ok(())
}

fn exchange_headers<T, U>(mut stream: &mut U, caller_id: &str, service: &str) -> Result<(), Error>
    where T: ServicePair,
          U: std::io::Write + std::io::Read
{
    write_request::<T, U>(stream, caller_id, service)?;
    read_response::<T, U>(stream)
}
