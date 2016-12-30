use std::net::TcpStream;
use std::thread;
use std::collections::HashMap;
use std;
use super::error::Error;
use super::header::{encode, decode};
use super::Message;
use super::decoder::DecoderSource;
use super::encoder::Encoder;

pub struct Client<Treq: Message, Tres: Message> {
    caller_id: String,
    uri: String,
    service: String,
    phantom: std::marker::PhantomData<(Treq, Tres)>,
}

impl<Treq: Message, Tres: Message> Client<Treq, Tres> {
    pub fn new(caller_id: &str, uri: &str, service: &str) -> Client<Treq, Tres> {
        Client {
            caller_id: String::from(caller_id),
            uri: String::from(uri),
            service: String::from(service),
            phantom: std::marker::PhantomData,
        }
    }

    pub fn req(&self, args: &Treq) -> Result<Tres, Error> {
        Client::request_body(args, &self.uri, &self.caller_id, &self.service)
    }

    pub fn req_callback<F>(&self, args: Treq, callback: F)
        where F: Fn(Result<Tres, Error>) -> () + Send + 'static
    {
        let uri = self.uri.clone();
        let caller_id = self.caller_id.clone();
        let service = self.service.clone();
        thread::spawn(move || callback(Client::request_body(&args, &uri, &caller_id, &service)));
    }

    fn request_body(args: &Treq, uri: &str, caller_id: &str, service: &str) -> Result<Tres, Error> {
        let mut stream = TcpStream::connect(uri)?;
        exchange_headers::<Treq, _>(&mut stream, caller_id, service)?;

        let mut encoder = Encoder::new();
        args.encode(&mut encoder)?;
        encoder.write_to(&mut stream)?;

        let mut decoder = DecoderSource::new(&mut stream);
        let mut decoder = decoder.next().ok_or(Error::Mismatch)?;
        Tres::decode(&mut decoder)

    }
}

fn write_request<T: Message, U: std::io::Write>(mut stream: &mut U,
                                                caller_id: &str,
                                                service: &str)
                                                -> Result<(), Error> {
    let mut fields = HashMap::<String, String>::new();
    fields.insert(String::from("callerid"), String::from(caller_id));
    fields.insert(String::from("service"), String::from(service));
    fields.insert(String::from("md5sum"), T::md5sum());
    fields.insert(String::from("type"), T::msg_type());
    encode(fields, &mut stream)
}

fn header_matches(fields: &HashMap<String, String>) -> bool {
    fields.get("caller_id") != None
}

fn read_response<T: Message, U: std::io::Read>(mut stream: &mut U) -> Result<(), Error> {
    if header_matches(&decode(&mut stream)?) {
        Ok(())
    } else {
        Err(Error::Mismatch)
    }
}

fn exchange_headers<T, U>(mut stream: &mut U, caller_id: &str, service: &str) -> Result<(), Error>
    where T: Message,
          U: std::io::Write + std::io::Read
{
    write_request::<T, U>(stream, caller_id, service)?;
    read_response::<T, U>(stream)
}
