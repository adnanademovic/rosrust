extern crate rustc_serialize;

use rosxmlrpc;
use self::rustc_serialize::Encodable;
use std;
use std::error::Error as ErrorTrait;

pub struct Slave {
    server: rosxmlrpc::Server,
}

impl Slave {
    pub fn new(server_uri: &str) -> Slave {
        let server = rosxmlrpc::Server::new(server_uri, SlaveHandler {}).unwrap();
        Slave { server: server }
    }

    pub fn uri(&self) -> &str {
        return &self.server.uri;
    }
}

struct SlaveHandler {

}

type SerdeResult<T> = Result<T, Error>;

impl SlaveHandler {
    fn encode_response<T: Encodable>(response: SerdeResult<T>,
                                     message: &str,
                                     res: &mut rosxmlrpc::serde::Encoder<&mut Vec<u8>>)
                                     -> () {
        match response {
            Ok(value) => (1i32, message, value).encode(res).unwrap(),
            Err(err) => (-1i32, err.description(), 0).encode(res).unwrap(),
        }
    }

    fn param_update(&self, req: &mut rosxmlrpc::serde::Decoder) -> SerdeResult<i32> {
        let caller_id = try!(req.decode_request_parameter::<String>());
        let key = try!(req.decode_request_parameter::<String>());
        let value = try!(req.decode_request_parameter::<String>());
        if caller_id != "" && key != "" && value != "" {
            println!("{} {} {}", caller_id, key, value);
            Ok(0)
        } else {
            Err(Error::Protocol("Emtpy strings given".to_owned()))
        }
    }
}

impl rosxmlrpc::server::XmlRpcServer for SlaveHandler {
    fn handle(&self,
              method_name: &str,
              _: usize,
              req: &mut rosxmlrpc::serde::Decoder,
              res: &mut rosxmlrpc::serde::Encoder<&mut Vec<u8>>) {
        println!("METHOD CALL: {}", method_name);
        match method_name {
            "getBusStats" => unimplemented!(),
            "getBusInfo" => unimplemented!(),
            "getMasterUri" => unimplemented!(),
            "shutdown" => unimplemented!(),
            "getPid" => unimplemented!(),
            "getSubscriptions" => unimplemented!(),
            "getPublications" => unimplemented!(),
            "paramUpdate" => {
                SlaveHandler::encode_response(self.param_update(req), "Parameter updated", res)
            }
            "publisherUpdate" => unimplemented!(),
            "requestTopic" => unimplemented!(),
            _ => {}
        }
    }
}

#[derive(Debug)]
pub enum Error {
    Deserialization(rosxmlrpc::serde::decoder::Error),
    Protocol(String),
}

impl From<rosxmlrpc::serde::decoder::Error> for Error {
    fn from(err: rosxmlrpc::serde::decoder::Error) -> Error {
        Error::Deserialization(err)
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match *self {
            Error::Deserialization(ref err) => write!(f, "Deserialization error: {}", err),
            Error::Protocol(ref err) => write!(f, "Protocol error: {}", err),
        }
    }
}

impl std::error::Error for Error {
    fn description(&self) -> &str {
        match *self {
            Error::Deserialization(ref err) => err.description(),
            Error::Protocol(ref err) => &err,
        }
    }

    fn cause(&self) -> Option<&std::error::Error> {
        match *self {
            Error::Deserialization(ref err) => Some(err),
            Error::Protocol(..) => None,
        }
    }
}
