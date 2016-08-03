extern crate libc;
extern crate rustc_serialize;

use rosxmlrpc;
use self::rustc_serialize::Encodable;
use std;
use std::error::Error as ErrorTrait;
use self::libc::getpid;

pub struct Slave {
    server: rosxmlrpc::Server,
}

impl Slave {
    pub fn new(server_uri: &str) -> Result<Slave, Error> {
        let server = try!(rosxmlrpc::Server::new(server_uri,
                                                 SlaveHandler {
                                                     subscriptions: vec![],
                                                     publications: vec![],
                                                 }));
        Ok(Slave { server: server })
    }

    pub fn uri(&self) -> &str {
        return &self.server.uri;
    }
}

struct SlaveHandler {
    subscriptions: Vec<(String, String)>,
    publications: Vec<(String, String)>,
}

type SerdeResult<T> = Result<T, Error>;

impl SlaveHandler {
    fn encode_response<T: Encodable>(response: SerdeResult<T>,
                                     message: &str,
                                     res: &mut rosxmlrpc::serde::Encoder<&mut Vec<u8>>) {
        match response {
                Ok(value) => (1i32, message, value).encode(res),
                Err(err) => (-1i32, err.description(), 0).encode(res),
            }
            .unwrap_or_else(|err| {
                println!("Encoding error: {}", err);
            });
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

    fn get_pid(&self, req: &mut rosxmlrpc::serde::Decoder) -> SerdeResult<i32> {
        let caller_id = try!(req.decode_request_parameter::<String>());
        if caller_id != "" {
            Ok(unsafe { getpid() })
        } else {
            Err(Error::Protocol("Empty strings given".to_owned()))
        }
    }

    fn get_publications(&self,
                        req: &mut rosxmlrpc::serde::Decoder)
                        -> SerdeResult<&[(String, String)]> {
        let caller_id = try!(req.decode_request_parameter::<String>());
        if caller_id != "" {
            Ok(&self.publications)
        } else {
            Err(Error::Protocol("Empty strings given".to_owned()))
        }
    }

    fn get_subscriptions(&self,
                         req: &mut rosxmlrpc::serde::Decoder)
                         -> SerdeResult<&[(String, String)]> {
        let caller_id = try!(req.decode_request_parameter::<String>());
        if caller_id != "" {
            Ok(&self.subscriptions)
        } else {
            Err(Error::Protocol("Empty strings given".to_owned()))
        }
    }
}

impl rosxmlrpc::server::XmlRpcServer for SlaveHandler {
    fn handle(&self,
              method_name: &str,
              _: usize,
              req: &mut rosxmlrpc::serde::Decoder,
              res: &mut rosxmlrpc::serde::Encoder<&mut Vec<u8>>) {
        println!("CALLED METHOD: {}", method_name);
        match method_name {
            "getBusStats" => unimplemented!(),
            "getBusInfo" => unimplemented!(),
            "getMasterUri" => unimplemented!(),
            "shutdown" => unimplemented!(),
            "getPid" => {
                SlaveHandler::encode_response(self.get_pid(req), "PID", res);
            }
            "getSubscriptions" => {
                SlaveHandler::encode_response(self.get_subscriptions(req),
                                              "List of subscriptions",
                                              res);
            }
            "getPublications" => {
                SlaveHandler::encode_response(self.get_publications(req),
                                              "List of publications",
                                              res);
            }
            "paramUpdate" => {
                SlaveHandler::encode_response(self.param_update(req), "Parameter updated", res);
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
    Serialization(rosxmlrpc::serde::encoder::Error),
    XmlRpc(rosxmlrpc::error::Error),
}

impl From<rosxmlrpc::serde::decoder::Error> for Error {
    fn from(err: rosxmlrpc::serde::decoder::Error) -> Error {
        Error::Deserialization(err)
    }
}

impl From<rosxmlrpc::serde::encoder::Error> for Error {
    fn from(err: rosxmlrpc::serde::encoder::Error) -> Error {
        Error::Serialization(err)
    }
}

impl From<rosxmlrpc::error::Error> for Error {
    fn from(err: rosxmlrpc::error::Error) -> Error {
        Error::XmlRpc(err)
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match *self {
            Error::Deserialization(ref err) => write!(f, "Deserialization error: {}", err),
            Error::Protocol(ref err) => write!(f, "Protocol error: {}", err),
            Error::Serialization(ref err) => write!(f, "Serialization error: {}", err),
            Error::XmlRpc(ref err) => write!(f, "XML RPC error: {}", err),
        }
    }
}

impl std::error::Error for Error {
    fn description(&self) -> &str {
        match *self {
            Error::Deserialization(ref err) => err.description(),
            Error::Protocol(ref err) => &err,
            Error::Serialization(ref err) => err.description(),
            Error::XmlRpc(ref err) => err.description(),
        }
    }

    fn cause(&self) -> Option<&std::error::Error> {
        match *self {
            Error::Deserialization(ref err) => Some(err),
            Error::Protocol(..) => None,
            Error::Serialization(ref err) => Some(err),
            Error::XmlRpc(ref err) => Some(err),
        }
    }
}
