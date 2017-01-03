use nix::unistd::getpid;
use rosxmlrpc::{self, XmlRpcValue};
use rosxmlrpc::server::{Answer, ParameterIterator, XmlRpcServer};
use rustc_serialize::{Decodable, Encodable};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::sync::mpsc::Sender;
use super::error::{self, ErrorKind, Result};
use super::value::Topic;
use tcpros::{Publisher, Subscriber, Service};

pub struct SlaveHandler {
    pub subscriptions: Arc<Mutex<HashMap<String, Subscriber>>>,
    pub publications: Arc<Mutex<HashMap<String, Publisher>>>,
    pub services: Arc<Mutex<HashMap<String, Service>>>,
    hostname: String,
    shutdown_signal: Arc<Mutex<Sender<()>>>,
    master_uri: String,
    name: String,
}

impl XmlRpcServer for SlaveHandler {
    fn handle(&self, method_name: &str, mut req: ParameterIterator) -> Answer {
        info!("CALLED METHOD: {}", method_name);
        self.handle_call(method_name, &mut req).unwrap_or_else(|err| {
            // The call only fails if we fail to encode the response
            // That should never happen
            error!("Failed encoding response to XML-RPC request '{}': {}",
                   method_name,
                   err);
            panic!("Unacceptable failure to encode XML-RPC request")
        })
    }
}

impl SlaveHandler {
    pub fn new(master_uri: &str,
               hostname: &str,
               name: &str,
               shutdown_signal: Sender<()>)
               -> SlaveHandler {
        SlaveHandler {
            subscriptions: Arc::new(Mutex::new(HashMap::new())),
            publications: Arc::new(Mutex::new(HashMap::new())),
            services: Arc::new(Mutex::new(HashMap::new())),
            master_uri: String::from(master_uri),
            hostname: String::from(hostname),
            name: String::from(name),
            shutdown_signal: Arc::new(Mutex::new(shutdown_signal)),
        }
    }

    fn handle_call(&self,
                   method_name: &str,
                   req: &mut ParameterIterator)
                   -> error::rosxmlrpc::serde::Result<Answer> {
        match method_name {
            "getBusStats" => encode_response(self.get_bus_stats(req), "Bus stats"),
            "getBusInfo" => encode_response(self.get_bus_info(req), "Bus stats"),
            "getMasterUri" => encode_response(self.get_master_uri(req), "Master URI"),
            "shutdown" => encode_response(self.shutdown(req), "Shutdown"),
            "getPid" => encode_response(self.get_pid(req), "PID"),
            "getSubscriptions" => {
                encode_response(self.get_subscriptions(req), "List of subscriptions")
            }
            "getPublications" => {
                encode_response(self.get_publications(req), "List of publications")
            }
            "paramUpdate" => encode_response(self.param_update(req), "Parameter updated"),
            "publisherUpdate" => encode_response(self.publisher_update(req), "Publishers updated"),
            "requestTopic" => encode_response(self.request_topic(req), "Chosen protocol"),
            name => {
                encode_response::<i32>(Err(ErrorKind::Protocol(format!("Unimplemented method: \
                                                                        {}",
                                                                       name))
                                           .into()),
                                       "")
            }
        }
    }

    fn get_bus_stats(&self, req: &mut ParameterIterator) -> Result<BusStats> {
        let caller_id = pop::<String>(req)?;
        if caller_id == "" {
            bail!(ErrorKind::Protocol("Empty strings given".into()));
        }
        // TODO: implement actual stats displaying
        Ok(BusStats {
            publish: Vec::new(),
            subscribe: Vec::new(),
            service: ServiceStats {
                bytes_received: 0,
                bytes_sent: 0,
                number_of_requests: 0,
            },
        })
    }

    fn get_bus_info(&self, req: &mut ParameterIterator) -> Result<Vec<BusInfo>> {
        let caller_id = pop::<String>(req)?;
        if caller_id == "" {
            bail!(ErrorKind::Protocol("Empty strings given".into()));
        }
        // TODO: implement actual info displaying
        Ok(Vec::new())
    }

    fn param_update(&self, req: &mut ParameterIterator) -> Result<i32> {
        let caller_id = pop::<String>(req)?;
        let key = pop::<String>(req)?;
        // We don't do anything with parameter updates
        let value = req.next();
        if let None = value {
            bail!(ErrorKind::Protocol("Missing parameter".into()));
        }
        if caller_id == "" || key == "" {
            bail!(ErrorKind::Protocol("Empty strings given".into()));
        }
        Ok(0)
    }

    fn get_pid(&self, req: &mut ParameterIterator) -> Result<i32> {
        let caller_id = pop::<String>(req)?;
        if caller_id == "" {
            bail!(ErrorKind::Protocol("Empty strings given".into()));
        }
        Ok(getpid())
    }

    fn shutdown(&self, req: &mut ParameterIterator) -> Result<i32> {
        let caller_id = pop::<String>(req)?;
        let message = pop::<String>(req).unwrap_or(String::from(""));
        if caller_id == "" {
            bail!(ErrorKind::Protocol("Empty strings given".into()));
        }
        info!("Server is shutting down because: {}", message);
        if let Err(..) = self.shutdown_signal.lock().unwrap().send(()) {
            bail!(ErrorKind::Critical("Slave API is down already".into()));
        }
        Ok(0)
    }

    fn get_publications(&self, req: &mut ParameterIterator) -> Result<Vec<Topic>> {
        let caller_id = pop::<String>(req)?;
        if caller_id == "" {
            bail!(ErrorKind::Protocol("Empty strings given".into()));
        }
        Ok(self.publications
            .lock()
            .unwrap()
            .values()
            .map(|ref v| {
                return Topic {
                    name: v.topic.clone(),
                    datatype: v.msg_type.clone(),
                };
            })
            .collect())
    }

    fn get_subscriptions(&self, req: &mut ParameterIterator) -> Result<Vec<Topic>> {
        let caller_id = pop::<String>(req)?;
        if caller_id == "" {
            bail!(ErrorKind::Protocol("Empty strings given".into()));
        }
        Ok(self.subscriptions
            .lock()
            .unwrap()
            .values()
            .map(|ref v| {
                return Topic {
                    name: v.topic.clone(),
                    datatype: v.msg_type.clone(),
                };
            })
            .collect())
    }

    fn publisher_update(&self, req: &mut ParameterIterator) -> Result<i32> {
        let caller_id = pop::<String>(req)?;
        let topic = pop::<String>(req)?;
        let publishers = pop::<Vec<String>>(req)?;
        if caller_id == "" || topic == "" || publishers.iter().any(|ref x| x.as_str() == "") {
            bail!(ErrorKind::Protocol("Empty strings given".into()));
        }
        add_publishers_to_subscription(&mut self.subscriptions.lock().unwrap(),
                                       &self.name,
                                       &topic,
                                       publishers.into_iter())
            .and(Ok(0))
    }

    fn get_master_uri(&self, req: &mut ParameterIterator) -> Result<&str> {
        let caller_id = pop::<String>(req)?;
        if caller_id == "" {
            bail!(ErrorKind::Protocol("Empty strings given".into()));
        }
        Ok(&self.master_uri)
    }

    fn request_topic(&self, req: &mut ParameterIterator) -> Result<(String, String, i32)> {
        let caller_id = pop::<String>(req)?;
        let topic = pop::<String>(req)?;
        let protocols = req.next()
            .ok_or(ErrorKind::Protocol("Missing parameter".into()))?
            .value();
        let (ip, port) = match self.publications
            .lock()
            .unwrap()
            .get(&topic) {
            Some(publisher) => (self.hostname.clone(), publisher.port as i32),
            None => {
                bail!(ErrorKind::Protocol("Requested topic not published by node".into()));
            }
        };
        if caller_id == "" || topic == "" {
            bail!(ErrorKind::Protocol("Empty strings given".into()));
        }
        let protocols = match protocols {
            XmlRpcValue::Array(protocols) => protocols,
            _ => {
                bail!(ErrorKind::Protocol("Protocols need to be provided as [ [String, \
                                           XmlRpcLegalValue] ]"
                    .into()));
            }
        };
        let mut has_tcpros = false;
        for protocol in protocols {
            if let XmlRpcValue::Array(protocol) = protocol {
                if let Some(&XmlRpcValue::String(ref name)) = protocol.get(0) {
                    has_tcpros |= name == "TCPROS";
                }
            }
        }
        if has_tcpros {
            Ok((String::from("TCPROS"), ip, port))
        } else {
            Err(ErrorKind::Protocol("No matching protocols available".into()).into())
        }
    }
}

pub fn add_publishers_to_subscription<T>(subscriptions: &mut HashMap<String, Subscriber>,
                                         name: &str,
                                         topic: &str,
                                         publishers: T)
                                         -> Result<()>
    where T: Iterator<Item = String>
{
    if let Some(mut subscription) = subscriptions.get_mut(topic) {
        for publisher in publishers {
            if let Err(err) = connect_to_publisher(&mut subscription, &name, &publisher, &topic) {
                error!("ROS provided illegal publisher name '{}': {}",
                       publisher,
                       err);
                return Err(err);
            }
        }
    }
    Ok(())
}

fn encode_response<T: Encodable>(response: Result<T>,
                                 message: &str)
                                 -> error::rosxmlrpc::serde::Result<Answer> {
    use std::error::Error;
    let mut res = Answer::new();
    match response {
            Ok(value) => res.add(&(1i32, message, value)),
            Err(err) => res.add(&(-1i32, err.description(), 0)),
        }
        .map(|_| res)
}


fn pop<T: Decodable>(req: &mut ParameterIterator) -> Result<T> {
    req.next()
        .ok_or(ErrorKind::Protocol("Missing parameter".into()))?
        .read()
        .map_err(|v| ErrorKind::XmlRpc(error::rosxmlrpc::ErrorKind::Serde(v.into())).into())
}

fn connect_to_publisher(subscriber: &mut Subscriber,
                        caller_id: &str,
                        publisher: &str,
                        topic: &str)
                        -> Result<()> {
    let (protocol, hostname, port) = request_topic(publisher, caller_id, topic)?;
    if protocol != "TCPROS" {
        // This should never happen, due to the nature of ROS
        panic!("Expected TCPROS protocol from ROS publisher");
    }
    subscriber.connect_to((hostname.as_str(), port as u16)).map_err(|err| ErrorKind::Io(err).into())
}

fn request_topic(publisher_uri: &str,
                 caller_id: &str,
                 topic: &str)
                 -> error::rosxmlrpc::Result<(String, String, i32)> {
    let mut request = rosxmlrpc::client::Request::new("requestTopic");
    request.add(&caller_id)?;
    request.add(&topic)?;
    request.add(&[["TCPROS"]])?;
    let client = rosxmlrpc::Client::new(publisher_uri);
    let protocols = client.request::<(i32, String, (String, String, i32))>(request)?;
    Ok(protocols.2)
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
