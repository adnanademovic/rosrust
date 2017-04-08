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
    fn handle(&self,
              method_name: &str,
              mut req: ParameterIterator)
              -> error::rosxmlrpc::serde::Result<Answer> {
        info!("Slave API method called: {}", method_name);
        self.handle_call(method_name, &mut req)
    }
}

type HandleResult<T> = Result<::std::result::Result<T, String>>;

macro_rules! pop{
    ($src:expr; $t:ty) => ({
        match pop::<$t>($src)? {
            Ok(v) => v,
            Err(err) => return Ok(Err(err)),
        }
    })
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
            name => encode_response::<i32>(Ok(Err(format!("Unimplemented method: {}", name))), ""),
        }
    }

    fn get_bus_stats(&self, req: &mut ParameterIterator) -> HandleResult<BusStats> {
        let caller_id = pop!(req; String);
        if caller_id == "" {
            return Ok(Err("Empty strings given".into()));
        }
        // TODO: implement actual stats displaying
        Err("Method not implemented".into())
    }

    fn get_bus_info(&self, req: &mut ParameterIterator) -> HandleResult<Vec<BusInfo>> {
        let caller_id = pop!(req; String);
        if caller_id == "" {
            return Ok(Err("Empty strings given".into()));
        }
        // TODO: implement actual info displaying
        Err("Method not implemented".into())
    }

    fn param_update(&self, req: &mut ParameterIterator) -> HandleResult<i32> {
        let caller_id = pop!(req; String);
        let key = pop!(req; String);
        // We don't do anything with parameter updates
        let value = req.next();
        if let None = value {
            return Ok(Err("Missing parameter".into()));
        }
        if caller_id == "" || key == "" {
            return Ok(Err("Empty strings given".into()));
        }
        Ok(Ok(0))
    }

    fn get_pid(&self, req: &mut ParameterIterator) -> HandleResult<i32> {
        let caller_id = pop!(req; String);
        if caller_id == "" {
            return Ok(Err("Empty strings given".into()));
        }
        Ok(Ok(getpid()))
    }

    fn shutdown(&self, req: &mut ParameterIterator) -> HandleResult<i32> {
        let caller_id = pop!(req; String);
        let message = pop::<String>(req)
            .unwrap_or(Err("".into()))
            .unwrap_or("".into());
        if caller_id == "" {
            return Ok(Err("Empty strings given".into()));
        }
        info!("Server is shutting down because: {}", message);
        if let Err(..) = self.shutdown_signal
               .lock()
               .expect(FAILED_TO_LOCK)
               .send(()) {
            bail!("Slave API is down already");
        }
        Ok(Ok(0))
    }

    fn get_publications(&self, req: &mut ParameterIterator) -> HandleResult<Vec<Topic>> {
        let caller_id = pop!(req; String);
        if caller_id == "" {
            return Ok(Err("Empty strings given".into()));
        }
        Ok(Ok(self.publications
                  .lock()
                  .expect(FAILED_TO_LOCK)
                  .values()
                  .map(|ref v| {
                           return Topic {
                                      name: v.topic.clone(),
                                      datatype: v.msg_type.clone(),
                                  };
                       })
                  .collect()))
    }

    fn get_subscriptions(&self, req: &mut ParameterIterator) -> HandleResult<Vec<Topic>> {
        let caller_id = pop!(req; String);
        if caller_id == "" {
            return Ok(Err("Empty strings given".into()));
        }
        Ok(Ok(self.subscriptions
                  .lock()
                  .expect(FAILED_TO_LOCK)
                  .values()
                  .map(|ref v| {
                           return Topic {
                                      name: v.topic.clone(),
                                      datatype: v.msg_type.clone(),
                                  };
                       })
                  .collect()))
    }

    fn publisher_update(&self, req: &mut ParameterIterator) -> HandleResult<i32> {
        let caller_id = pop!(req; String);
        let topic = pop!(req; String);
        let publishers = pop!(req; Vec<String>);
        if caller_id == "" || topic == "" || publishers.iter().any(|ref x| x.as_str() == "") {
            return Ok(Err("Empty strings given".into()));
        }
        add_publishers_to_subscription(&mut self.subscriptions.lock().expect(FAILED_TO_LOCK),
                                       &self.name,
                                       &topic,
                                       publishers.into_iter())
                .and(Ok(Ok(0)))
    }

    fn get_master_uri(&self, req: &mut ParameterIterator) -> HandleResult<&str> {
        let caller_id = pop!(req; String);
        if caller_id == "" {
            return Ok(Err("Empty strings given".into()));
        }
        Ok(Ok(&self.master_uri))
    }

    fn request_topic(&self, req: &mut ParameterIterator) -> HandleResult<(String, String, i32)> {
        let caller_id = pop!(req; String);
        let topic = pop!(req; String);
        let protocols = match req.next() {
            Some(v) => v.value(),
            None => return Ok(Err("Missing parameter".into())),
        };
        let (ip, port) = match self.publications
                  .lock()
                  .expect(FAILED_TO_LOCK)
                  .get(&topic) {
            Some(publisher) => (self.hostname.clone(), publisher.port as i32),
            None => {
                return Ok(Err("Requested topic not published by node".into()));
            }
        };
        if caller_id == "" || topic == "" {
            return Ok(Err("Empty strings given".into()));
        }
        let protocols = match protocols {
            XmlRpcValue::Array(protocols) => protocols,
            _ => {
                return Ok(Err("Protocols need to be provided as [ [String, \
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
            Ok(Ok((String::from("TCPROS"), ip, port)))
        } else {
            Ok(Err("No matching protocols available".into()))
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
                let info = err.iter()
                    .map(|v| format!("{}", v))
                    .collect::<Vec<_>>()
                    .join("\nCaused by:");
                error!("Failed to connect to publisher '{}': {}", publisher, info);
                return Err(err);
            }
        }
    }
    Ok(())
}

fn encode_response<T: Encodable>(response: HandleResult<T>,
                                 message: &str)
                                 -> error::rosxmlrpc::serde::Result<Answer> {
    use std::error::Error;
    let mut res = Answer::new();
    match response {
            Ok(value) => {
                match value {
                    // Success
                    Ok(value) => res.add(&(1i32, message, value)),
                    // Bad request provided
                    Err(err) => res.add(&(-1i32, err, 0)),
                }
            }
            // System failure while handling request
            Err(err) => res.add(&(0i32, err.description(), 0)),
        }
        .map(|_| res)
}


fn pop<T: Decodable>(req: &mut ParameterIterator) -> HandleResult<T> {
    Ok(Ok(match req.next() {
                  Some(v) => v,
                  None => return Ok(Err("Missing parameter".into())),
              }
              .read::<T>()
              .map_err(|v| error::rosxmlrpc::Error::from(v))?))
}

fn connect_to_publisher(subscriber: &mut Subscriber,
                        caller_id: &str,
                        publisher: &str,
                        topic: &str)
                        -> Result<()> {
    let (protocol, hostname, port) = request_topic(publisher, caller_id, topic)?;
    if protocol != "TCPROS" {
        bail!("Publisher responded with a non-TCPROS protocol: {}",
              protocol)
    }
    subscriber
        .connect_to((hostname.as_str(), port as u16))
        .map_err(|err| ErrorKind::Io(err).into())
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
    let protocols = client
        .request::<(i32, String, (String, String, i32))>(request)?;
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

static FAILED_TO_LOCK: &'static str = "Failed to acquire lock";
