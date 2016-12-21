use rosxmlrpc;
use rosxmlrpc::XmlRpcValue;
use rustc_serialize::{Decodable, Encodable};
use std::collections::HashMap;
use std::sync::mpsc::{Sender, Receiver};
use std::sync::mpsc;
use std::sync::Mutex;
use std::error::Error as ErrorTrait;
use nix::unistd::getpid;
use super::error::ServerError as Error;
use super::value::Topic;
use tcpros::{self, Message, Publisher};

struct Subscription {
    topic: String,
    msg_type: String,
    channel: Sender<String>,
}

pub struct Slave {
    server: rosxmlrpc::Server,
    req: Mutex<Receiver<(String, rosxmlrpc::server::ParameterIterator)>>,
    res: Mutex<Sender<rosxmlrpc::server::Answer>>,
    subscriptions: HashMap<String, Subscription>,
    publications: HashMap<String, Publisher>,
    master_uri: String,
}

struct SlaveHandler {
    req: Mutex<Sender<(String, rosxmlrpc::server::ParameterIterator)>>,
    res: Mutex<Receiver<rosxmlrpc::server::Answer>>,
}

type SerdeResult<T> = Result<T, Error>;

impl Slave {
    pub fn new(master_uri: &str, server_uri: &str) -> Result<Slave, Error> {
        let (tx_req, rx_req) = mpsc::channel();
        let (tx_res, rx_res) = mpsc::channel();
        let server = rosxmlrpc::Server::new(server_uri,
                                            SlaveHandler {
                                                req: Mutex::new(tx_req),
                                                res: Mutex::new(rx_res),
                                            })?;
        Ok(Slave {
            server: server,
            subscriptions: HashMap::new(),
            publications: HashMap::new(),
            master_uri: master_uri.to_owned(),
            req: Mutex::new(rx_req),
            res: Mutex::new(tx_res),
        })
    }

    pub fn uri(&self) -> &str {
        return &self.server.uri;
    }

    fn get_bus_stats(&self,
                     req: &mut rosxmlrpc::server::ParameterIterator)
                     -> SerdeResult<BusStats> {
        let caller_id = pop::<String>(req)?;
        if caller_id != "" {
            // TODO: implement actual stats displaying
            Ok(BusStats {
                publish: vec![],
                subscribe: vec![],
                service: ServiceStats {
                    bytes_received: 0,
                    bytes_sent: 0,
                    number_of_requests: 0,
                },
            })
        } else {
            Err(Error::Protocol("Empty strings given".to_owned()))
        }
    }

    fn get_bus_info(&self,
                    req: &mut rosxmlrpc::server::ParameterIterator)
                    -> SerdeResult<Vec<BusInfo>> {
        let caller_id = pop::<String>(req)?;
        if caller_id != "" {
            // TODO: implement actual info displaying
            Ok(vec![])
        } else {
            Err(Error::Protocol("Empty strings given".to_owned()))
        }
    }

    fn encode_response<T: Encodable>(&self,
                                     response: SerdeResult<T>,
                                     message: &str)
                                     -> SerdeResult<()> {
        let mut res = rosxmlrpc::server::Answer::new();
        match response {
            Ok(value) => res.add(&(1i32, message, value)),
            Err(err) => res.add(&(-1i32, err.description(), 0)),
        }?;

        self.res.lock().unwrap().send(res).unwrap();
        Ok(())
    }

    fn param_update(&self, req: &mut rosxmlrpc::server::ParameterIterator) -> SerdeResult<i32> {
        let caller_id = pop::<String>(req)?;
        let key = pop::<String>(req)?;
        let value = req.next()
            .ok_or(Error::Protocol(String::from("Missing parameter")))?
            .value();
        if caller_id != "" && key != "" {
            // TODO: implement handling of parameter updates
            println!("{} {} {}", caller_id, key, value);
            Ok(0)
        } else {
            Err(Error::Protocol("Emtpy strings given".to_owned()))
        }
    }

    fn get_pid(&self, req: &mut rosxmlrpc::server::ParameterIterator) -> SerdeResult<i32> {
        let caller_id = pop::<String>(req)?;
        if caller_id != "" {
            Ok(getpid())
        } else {
            Err(Error::Protocol("Empty strings given".to_owned()))
        }
    }

    fn shutdown(&mut self, req: &mut rosxmlrpc::server::ParameterIterator) -> SerdeResult<i32> {
        let caller_id = pop::<String>(req)?;
        let message = pop::<String>(req).unwrap_or(String::from(""));
        if caller_id != "" {
            println!("Server is shutting down because: {}", message);
            match self.server.shutdown() {
                Ok(()) => Ok(0),
                Err(_) => Err(Error::Critical("Failed to shutdown server".to_owned())),
            }
        } else {
            Err(Error::Protocol("Empty strings given".to_owned()))
        }
    }

    pub fn add_publication<T>(&mut self, hostname: &str, topic: &str) -> Result<(), tcpros::Error>
        where T: Message
    {
        use std::collections::hash_map::Entry;
        match self.publications.entry(String::from(topic)) {
            Entry::Occupied(entry) => {
                let publisher = entry.get();
                if publisher.msg_type != T::msg_type() {
                    Err(tcpros::Error::Mismatch)
                } else {
                    Ok(())
                }
            }
            Entry::Vacant(entry) => {
                let publisher = Publisher::new::<T, _>(format!("{}:0", hostname).as_str(), topic)?;
                entry.insert(publisher);
                Ok(())
            }
        }
    }

    pub fn get_publication<T>(&mut self, topic: &str) -> Option<&Publisher>
        where T: Message
    {
        self.publications.get(topic)
    }

    pub fn remove_publication(&mut self, topic: &str) {
        self.publications.remove(topic);
    }

    fn get_publications(&self,
                        req: &mut rosxmlrpc::server::ParameterIterator)
                        -> SerdeResult<Vec<Topic>> {
        let caller_id = pop::<String>(req)?;
        if caller_id != "" {
            Ok(self.publications
                .values()
                .map(|ref v| {
                    return Topic {
                        name: v.topic.clone(),
                        datatype: v.msg_type.clone(),
                    };
                })
                .collect())
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
                         req: &mut rosxmlrpc::server::ParameterIterator)
                         -> SerdeResult<Vec<Topic>> {
        let caller_id = pop::<String>(req)?;
        if caller_id != "" {
            Ok(self.subscriptions
                .values()
                .map(|ref v| {
                    return Topic {
                        name: v.topic.clone(),
                        datatype: v.msg_type.clone(),
                    };
                })
                .collect())
        } else {
            Err(Error::Protocol("Empty strings given".to_owned()))
        }
    }

    fn publisher_update(&self, req: &mut rosxmlrpc::server::ParameterIterator) -> SerdeResult<i32> {
        let caller_id = pop::<String>(req)?;
        let topic = pop::<String>(req)?;
        let publishers = pop::<Vec<String>>(req)?;
        if caller_id != "" && topic != "" && publishers.iter().all(|ref x| x.as_str() != "") {
            if let Some(subscription) = self.subscriptions.get(&topic) {
                for publisher in publishers {
                    subscription.channel
                        .send(publisher)
                        .or(Err(Error::Protocol("Unable to accept publisher".to_owned())))?
                }
            }
            Ok(0)
        } else {
            Err(Error::Protocol("Empty strings given".to_owned()))
        }
    }

    fn get_master_uri(&self, req: &mut rosxmlrpc::server::ParameterIterator) -> SerdeResult<&str> {
        let caller_id = pop::<String>(req)?;
        if caller_id != "" {
            Ok(&self.master_uri)
        } else {
            Err(Error::Protocol("Empty strings given".to_owned()))
        }
    }

    fn request_topic(&self,
                     req: &mut rosxmlrpc::server::ParameterIterator)
                     -> SerdeResult<(String, String, i32)> {
        let caller_id = pop::<String>(req)?;
        let topic = pop::<String>(req)?;
        let protocols = req.next()
            .ok_or(Error::Protocol(String::from("Missing parameter")))?
            .value();
        let publisher = self.publications
            .get(&topic)
            .ok_or(Error::Protocol("Requested topic not published by node".to_owned()))?;
        if caller_id != "" && topic != "" {
            if let XmlRpcValue::Array(protocols) = protocols {
                let mut has_tcpros = false;
                for protocol in protocols {
                    if let XmlRpcValue::Array(protocol) = protocol {
                        if let Some(&XmlRpcValue::String(ref name)) = protocol.get(0) {
                            has_tcpros |= name == "TCPROS";
                        }
                    }
                }
                if has_tcpros {
                    Ok(("TCPROS".to_owned(), publisher.ip.clone(), publisher.port as i32))
                } else {
                    Err(Error::Protocol("No matching protocols available".to_owned()))
                }
            } else {
                Err(Error::Protocol("Protocols need to be provided as [ [String, \
                                     XmlRpcLegalValue] ]"
                    .to_owned()))
            }
        } else {
            Err(Error::Protocol("Empty parameters given".to_owned()))
        }
    }

    fn handle_call(&mut self,
                   method_name: &str,
                   req: &mut rosxmlrpc::server::ParameterIterator)
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
                Ok((method_name, mut req)) => self.handle_call(&method_name, &mut req)?,
            }
        }
    }
}

impl rosxmlrpc::server::XmlRpcServer for SlaveHandler {
    fn handle(&self,
              method_name: &str,
              req: rosxmlrpc::server::ParameterIterator)
              -> rosxmlrpc::server::Answer {
        println!("CALLED METHOD: {}", method_name);
        self.req.lock().unwrap().send((method_name.to_owned(), req)).unwrap();
        self.res.lock().unwrap().recv().unwrap()
    }
}

fn pop<T: Decodable>(req: &mut rosxmlrpc::server::ParameterIterator) -> SerdeResult<T> {
    req.next()
        .ok_or(Error::Protocol(String::from("Missing parameter")))?
        .read()
        .map_err(|v| Error::Decoding(v))
}

#[derive(RustcEncodable)]
pub struct BusStats {
    pub publish: Vec<PublishStats>,
    pub subscribe: Vec<SubscribeStats>,
    pub service: ServiceStats,
}

#[derive(RustcEncodable)]
pub struct PublishStats {
    pub name: String,
    pub data_sent: String,
    pub connection_data: PublishConnectionData,
}

#[derive(RustcEncodable)]
pub struct PublishConnectionData {
    pub connection_id: String,
    pub bytes_sent: i32,
    pub number_sent: i32,
    pub connected: bool,
}

#[derive(RustcEncodable)]
pub struct SubscribeStats {
    pub name: String,
    pub connection_data: SubscribeConnectionData,
}

#[derive(RustcEncodable)]
pub struct SubscribeConnectionData {
    pub connection_id: String,
    pub bytes_received: i32,
    pub drop_estimate: i32,
    pub connected: bool,
}

#[derive(RustcEncodable)]
pub struct ServiceStats {
    pub number_of_requests: i32,
    pub bytes_received: i32,
    pub bytes_sent: i32,
}

#[derive(RustcEncodable)]
pub struct BusInfo {
    pub connection_id: String,
    pub destination_id: String,
    pub direction: String,
    pub transport: String,
    pub topic: String,
    pub connected: bool,
}
