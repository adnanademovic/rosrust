use rosxmlrpc;
use rustc_serialize::Encodable;
use std::collections::HashMap;
use std::sync::mpsc::{Sender, Receiver};
use std::sync::mpsc;
use std::sync::Mutex;
use std::error::Error as ErrorTrait;
use libc::getpid;
use super::error::ServerError as Error;

struct Subscription {
    topic: String,
    msg_type: String,
    channel: Sender<String>,
}

pub struct Slave {
    server: rosxmlrpc::Server,
    req: Mutex<Receiver<(String, rosxmlrpc::serde::Decoder)>>,
    res: Mutex<Sender<Vec<u8>>>,
    subscriptions: HashMap<String, Subscription>,
    pub publications: Vec<(String, String)>,
    master_uri: String,
}

struct SlaveHandler {
    req: Mutex<Sender<(String, rosxmlrpc::serde::Decoder)>>,
    res: Mutex<Receiver<Vec<u8>>>,
}

type SerdeResult<T> = Result<T, Error>;

impl Slave {
    pub fn new(master_uri: &str, server_uri: &str) -> Result<Slave, Error> {
        let (tx_req, rx_req) = mpsc::channel();
        let (tx_res, rx_res) = mpsc::channel();
        let server = try!(rosxmlrpc::Server::new(server_uri,
                                                 SlaveHandler {
                                                     req: Mutex::new(tx_req),
                                                     res: Mutex::new(rx_res),
                                                 }));
        Ok(Slave {
            server: server,
            subscriptions: HashMap::new(),
            publications: vec![],
            master_uri: master_uri.to_owned(),
            req: Mutex::new(rx_req),
            res: Mutex::new(tx_res),
        })
    }

    pub fn uri(&self) -> &str {
        return &self.server.uri;
    }

    fn get_bus_stats(&self,
                     req: &mut rosxmlrpc::serde::Decoder)
                     -> SerdeResult<(&[(String, String, (String, i32, i32, bool))],
                                     &[(String, (String, i32, i32, bool))],
                                     (i32, i32, i32))> {
        let caller_id = try!(req.decode_request_parameter::<String>());
        if caller_id != "" {
            // TODO: implement actual stats displaying
            Ok((&[], &[], (0, 0, 0)))
        } else {
            Err(Error::Protocol("Empty strings given".to_owned()))
        }
    }

    fn get_bus_info(&self,
                    req: &mut rosxmlrpc::serde::Decoder)
                    -> SerdeResult<&[(String, String, String, String, String, bool)]> {
        let caller_id = try!(req.decode_request_parameter::<String>());
        if caller_id != "" {
            // TODO: implement actual info displaying
            Ok(&[])
        } else {
            Err(Error::Protocol("Empty strings given".to_owned()))
        }
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
            // TODO: implement handling of parameter updates
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

    pub fn add_subscription(&mut self, topic: &str, msg_type: &str) -> Option<Receiver<String>> {
        let (tx, rx) = mpsc::channel();
        if self.subscriptions.contains_key(topic) {
            None
        } else {
            self.subscriptions.insert(topic.to_owned(),
                                      Subscription {
                                          topic: topic.to_owned(),
                                          msg_type: msg_type.to_owned(),
                                          channel: tx,
                                      });
            Some(rx)
        }
    }

    pub fn remove_subscription(&mut self, topic: &str) {
        self.subscriptions.remove(topic);
    }

    fn get_subscriptions(&self,
                         req: &mut rosxmlrpc::serde::Decoder)
                         -> SerdeResult<Vec<(String, String)>> {
        let caller_id = try!(req.decode_request_parameter::<String>());
        if caller_id != "" {
            Ok(self.subscriptions
                .values()
                .map(|ref v| return (v.topic.clone(), v.msg_type.clone()))
                .collect())
        } else {
            Err(Error::Protocol("Empty strings given".to_owned()))
        }
    }

    fn publisher_update(&self, req: &mut rosxmlrpc::serde::Decoder) -> SerdeResult<i32> {
        let caller_id = try!(req.decode_request_parameter::<String>());
        let topic = try!(req.decode_request_parameter::<String>());
        let publishers = try!(req.decode_request_parameter::<Vec<String>>());
        if caller_id != "" && topic != "" && publishers.iter().all(|ref x| x.as_str() != "") {
            if let Some(subscription) = self.subscriptions.get(&topic) {
                for publisher in publishers {
                    try!(subscription.channel
                        .send(publisher)
                        .or(Err(Error::Protocol("Unable to accept publisher".to_owned()))))
                }
            }
            Ok(0)
        } else {
            Err(Error::Protocol("Empty strings given".to_owned()))
        }
    }

    fn get_master_uri(&self, req: &mut rosxmlrpc::serde::Decoder) -> SerdeResult<&str> {
        let caller_id = try!(req.decode_request_parameter::<String>());
        if caller_id != "" {
            Ok(&self.master_uri)
        } else {
            Err(Error::Protocol("Empty strings given".to_owned()))
        }
    }

    fn request_topic(&self, req: &mut rosxmlrpc::serde::Decoder) -> SerdeResult<&[String]> {
        let caller_id = try!(req.decode_request_parameter::<String>());
        let topic = try!(req.decode_request_parameter::<String>());
        let protocols = try!(req.decode_request_parameter::<Vec<String>>());
        if caller_id != "" && topic != "" && !protocols.is_empty() &&
           protocols.iter().all(|ref x| !x.is_empty()) {
            // TODO: We don't actually implement any protocols (yet).
            //       The response will always be empty for now.
            Ok(&[])
        } else {
            Err(Error::Protocol("Empty parameters given".to_owned()))
        }
    }

    fn handle_call(&mut self,
                   method_name: &str,
                   req: &mut rosxmlrpc::serde::Decoder)
                   -> SerdeResult<()> {
        println!("HANDLING METHOD: {}", method_name);
        match method_name {
            "getBusStats" => self.encode_response(self.get_bus_stats(req), "Bus stats"),
            "getBusInfo" => self.encode_response(self.get_bus_info(req), "Bus stats"),
            "getMasterUri" => self.encode_response(self.get_master_uri(req), "Master URI"),
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
            "publisherUpdate" => {
                self.encode_response(self.publisher_update(req), "Publishers updated")
            }
            "requestTopic" => self.encode_response(self.request_topic(req), "Chosen protocol"),
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
