extern crate libc;
extern crate rustc_serialize;

use rosxmlrpc;
use self::rustc_serialize::Encodable;
use std;
use std::sync::mpsc::{Sender, Receiver};
use std::sync::mpsc;
use std::sync::Mutex;
use std::error::Error as ErrorTrait;
use self::libc::getpid;

pub struct Slave {
    server: rosxmlrpc::Server,
    req: Mutex<Receiver<(String, rosxmlrpc::serde::Decoder)>>,
    res: Mutex<Sender<Vec<u8>>>,
    subscriptions: Vec<(String, String)>,
    publications: Vec<(String, String)>,
}

struct SlaveHandler {
    req: Mutex<Sender<(String, rosxmlrpc::serde::Decoder)>>,
    res: Mutex<Receiver<Vec<u8>>>,
}

type SerdeResult<T> = Result<T, Error>;

impl Slave {
    pub fn new(server_uri: &str) -> Result<Slave, Error> {
        let (tx_req, rx_req) = mpsc::channel();
        let (tx_res, rx_res) = mpsc::channel();
        let server = try!(rosxmlrpc::Server::new(server_uri,
                                                 SlaveHandler {
                                                     req: Mutex::new(tx_req),
                                                     res: Mutex::new(rx_res),
                                                 }));
        Ok(Slave {
            server: server,
            subscriptions: vec![],
            publications: vec![],
            req: Mutex::new(rx_req),
            res: Mutex::new(tx_res),
        })
    }

    pub fn uri(&self) -> &str {
        return &self.server.uri;
    }

    fn encode_response<T: Encodable>(&self,
                                     response: SerdeResult<T>,
                                     message: &str)
                                     -> SerdeResult<()> {
        let mut body = Vec::<u8>::new();
        {
            let mut res = rosxmlrpc::serde::Encoder::new(&mut body);
            try!(res.start_response());
            try!(match response {
                Ok(value) => (1i32, message, value).encode(&mut res),
                Err(err) => (-1i32, err.description(), 0).encode(&mut res),
            });
            try!(res.end_response());
        }
        self.res.lock().unwrap().send(body).unwrap();
        Ok(())
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

    fn shutdown(&mut self, req: &mut rosxmlrpc::serde::Decoder) -> SerdeResult<i32> {
        let caller_id = try!(req.decode_request_parameter::<String>());
        if caller_id != "" {
            match self.server.shutdown() {
                Ok(()) => Ok(0),
                Err(_) => Err(Error::Critical("Failed to shutdown server".to_owned())),
            }
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

    fn handle_call(&mut self,
                   method_name: &str,
                   req: &mut rosxmlrpc::serde::Decoder)
                   -> SerdeResult<()> {
        println!("HANDLING METHOD: {}", method_name);
        match method_name {
            "getBusStats" => unimplemented!(),
            "getBusInfo" => unimplemented!(),
            "getMasterUri" => unimplemented!(),
            "shutdown" => {
                let data = self.shutdown(req);
                self.encode_response(data, "Shutdown")
            }
            "getPid" => self.encode_response(self.get_pid(req), "PID"),
            "getSubscriptions" => {
                self.encode_response(self.get_subscriptions(req), "List of subscriptions")
            }
            "getPublications" => {
                self.encode_response(self.get_publications(req), "List of publications")
            }
            "paramUpdate" => self.encode_response(self.param_update(req), "Parameter updated"),
            "publisherUpdate" => unimplemented!(),
            "requestTopic" => unimplemented!(),
            name => {
                self.encode_response::<i32>(Err(Error::Protocol(format!("Unimplemented method: \
                                                                         {}",
                                                                        name))),
                                            "")
            }
        }
    }

    pub fn handle_calls(&mut self) -> Result<(), String> {
        loop {
            let recv = self.req.lock().unwrap().recv();
            match recv {
                Err(_) => return Ok(()),
                Ok((method_name, mut req)) => {
                    if let Err(err) = self.handle_call(&method_name, &mut req) {
                        match err {
                            Error::Critical(msg) => {
                                return Err(msg);
                            }
                            _ => {
                                println!("{}", err);
                            }
                        }
                    }
                }
            }
        }
    }

    pub fn handle_call_queue(&mut self) -> Result<mpsc::TryRecvError, Error> {
        loop {
            let recv = self.req.lock().unwrap().try_recv();
            match recv {
                Err(err) => return Ok(err),
                Ok((method_name, mut req)) => try!(self.handle_call(&method_name, &mut req)),
            }
        }
    }
}

impl rosxmlrpc::server::XmlRpcServer for SlaveHandler {
    fn handle(&self, method_name: &str, _: usize, req: rosxmlrpc::serde::Decoder) -> Vec<u8> {
        println!("CALLED METHOD: {}", method_name);
        self.req.lock().unwrap().send((method_name.to_owned(), req)).unwrap();
        self.res.lock().unwrap().recv().unwrap()
    }
}

#[derive(Debug)]
pub enum Error {
    Deserialization(rosxmlrpc::serde::decoder::Error),
    Protocol(String),
    Critical(String),
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
            Error::Critical(ref err) => write!(f, "Critical error: {}", err),
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
            Error::Critical(ref err) => &err,
            Error::Serialization(ref err) => err.description(),
            Error::XmlRpc(ref err) => err.description(),
        }
    }

    fn cause(&self) -> Option<&std::error::Error> {
        match *self {
            Error::Deserialization(ref err) => Some(err),
            Error::Protocol(..) => None,
            Error::Critical(..) => None,
            Error::Serialization(ref err) => Some(err),
            Error::XmlRpc(ref err) => Some(err),
        }
    }
}
