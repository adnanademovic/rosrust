use nix::unistd::getpid;
use rosxmlrpc::{Response, ResponseError, Server};
use std::net::SocketAddr;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use futures::sync::mpsc::Sender;
use super::error::{self, ErrorKind, Result};
use tcpros::{Publisher, Service, Subscriber};
use xml_rpc::{self, Params, Value};

pub struct SlaveHandler {
    pub subscriptions: Arc<Mutex<HashMap<String, Subscriber>>>,
    pub publications: Arc<Mutex<HashMap<String, Publisher>>>,
    pub services: Arc<Mutex<HashMap<String, Service>>>,
    server: Server,
}

fn unwrap_array_case(params: Params) -> Params {
    if let Some(&Value::Array(ref items)) = params.get(0) {
        return items.clone();
    }
    params
}

impl SlaveHandler {
    pub fn new(
        master_uri: &str,
        hostname: &str,
        name: &str,
        shutdown_signal: Sender<()>,
    ) -> SlaveHandler {
        use futures::Sink;

        let mut server = Server::default();

        server.register_value("getBusStats", "Bus stats", |_args| {
            // TODO: implement actual stats displaying
            Err(ResponseError::Server("Method not implemented".into()))
        });

        server.register_value("getBusInfo", "Bus info", |_args| {
            // TODO: implement actual info displaying
            Err(ResponseError::Server("Method not implemented".into()))
        });

        let master_uri_string = String::from(master_uri);

        server.register_value("getMasterUri", "Master URI", move |_args| {
            Ok(Value::String(master_uri_string.clone()))
        });

        server.register_value("shutdown", "Shutdown", move |args| {
            let mut args = unwrap_array_case(args).into_iter();
            let _caller_id = args.next()
                .ok_or_else(|| ResponseError::Client("Missing argument 'caller_id'".into()))?;
            let message = match args.next() {
                Some(Value::String(message)) => message,
                _ => return Err(ResponseError::Client("Missing argument 'message'".into())),
            };
            info!("Server is shutting down because: {}", message);
            match shutdown_signal.clone().wait().send(()) {
                Ok(()) => Ok(Value::Int(0)),
                Err(err) => {
                    error!("Shutdown error: {:?}", err);
                    Err(ResponseError::Server("Failed to shut down".into()))
                }
            }
        });

        server.register_value("getPid", "PID", |_args| Ok(Value::Int(getpid().into())));

        let subscriptions = Arc::new(Mutex::new(HashMap::<String, Subscriber>::new()));
        let subs = Arc::clone(&subscriptions);

        server.register_value("getSubscriptions", "List of subscriptions", move |_args| {
            Ok(Value::Array(
                subs.lock()
                    .expect(FAILED_TO_LOCK)
                    .values()
                    .map(|v| {
                        Value::Array(vec![
                            Value::String(v.topic.clone()),
                            Value::String(v.msg_type.clone()),
                        ])
                    })
                    .collect(),
            ))
        });

        let publications = Arc::new(Mutex::new(HashMap::<String, Publisher>::new()));
        let pubs = Arc::clone(&publications);

        server.register_value("getPublications", "List of publications", move |_args| {
            Ok(Value::Array(
                pubs.lock()
                    .expect(FAILED_TO_LOCK)
                    .values()
                    .map(|v| {
                        Value::Array(vec![
                            Value::String(v.topic.clone()),
                            Value::String(v.msg_type.clone()),
                        ])
                    })
                    .collect(),
            ))
        });

        server.register_value("paramUpdate", "Parameter updated", |_args| {
            // We don't do anything with parameter updates
            Ok(Value::Int(0))
        });

        let name_string = String::from(name);
        let subs = Arc::clone(&subscriptions);

        server.register_value("publisherUpdate", "Publishers updated", move |args| {
            let mut args = unwrap_array_case(args).into_iter();
            let _caller_id = args.next()
                .ok_or_else(|| ResponseError::Client("Missing argument 'caller_id'".into()))?;
            let topic = match args.next() {
                Some(Value::String(topic)) => topic,
                _ => return Err(ResponseError::Client("Missing argument 'topic'".into())),
            };
            let publishers = match args.next() {
                Some(Value::Array(publishers)) => publishers,
                _ => {
                    return Err(ResponseError::Client(
                        "Missing argument 'publishers'".into(),
                    ))
                }
            };
            let publishers = publishers
                .into_iter()
                .map(|v| match v {
                    Value::String(x) => Ok(x),
                    _ => Err(ResponseError::Client(
                        "Publishers need to be strings".into(),
                    )),
                })
                .collect::<Response<Vec<String>>>()?;

            add_publishers_to_subscription(
                &mut subs.lock().expect(FAILED_TO_LOCK),
                &name_string,
                &topic,
                publishers.into_iter(),
            ).map_err(|v| ResponseError::Server(format!("Failed to handle publishers: {}", v)))?;
            Ok(Value::Int(0))
        });

        let hostname_string = String::from(hostname);
        let pubs = Arc::clone(&publications);

        server.register_value("requestTopic", "Chosen protocol", move |args| {
            let mut args = unwrap_array_case(args).into_iter();
            let _caller_id = args.next()
                .ok_or_else(|| ResponseError::Client("Missing argument 'caller_id'".into()))?;
            let topic = match args.next() {
                Some(Value::String(topic)) => topic,
                _ => return Err(ResponseError::Client("Missing argument 'topic'".into())),
            };
            let protocols = match args.next() {
                Some(Value::Array(protocols)) => protocols,
                Some(_) => {
                    return Err(ResponseError::Client(
                        "Protocols need to be provided as [ [String, XmlRpcLegalValue] ]".into(),
                    ))
                }
                None => return Err(ResponseError::Client("Missing argument 'protocols'".into())),
            };
            let (ip, port) = match pubs.lock().expect(FAILED_TO_LOCK).get(&topic) {
                Some(publisher) => (hostname_string.clone(), i32::from(publisher.port)),
                None => {
                    return Err(ResponseError::Client(
                        "Requested topic not published by node".into(),
                    ));
                }
            };
            let mut has_tcpros = false;
            for protocol in protocols {
                if let Value::Array(protocol) = protocol {
                    if let Some(&Value::String(ref name)) = protocol.get(0) {
                        has_tcpros |= name == "TCPROS";
                    }
                }
            }
            if has_tcpros {
                Ok(Value::Array(vec![
                    Value::String("TCPROS".into()),
                    Value::String(ip),
                    Value::Int(port),
                ]))
            } else {
                Err(ResponseError::Server(
                    "No matching protocols available".into(),
                ))
            }
        });

        SlaveHandler {
            subscriptions: subscriptions,
            publications: publications,
            services: Arc::new(Mutex::new(HashMap::new())),
            server: server,
        }
    }

    pub fn bind(self, addr: &SocketAddr) -> xml_rpc::error::Result<xml_rpc::server::BoundServer> {
        self.server.bind(addr)
    }
}

pub fn add_publishers_to_subscription<T>(
    subscriptions: &mut HashMap<String, Subscriber>,
    name: &str,
    topic: &str,
    publishers: T,
) -> Result<()>
where
    T: Iterator<Item = String>,
{
    if let Some(mut subscription) = subscriptions.get_mut(topic) {
        for publisher in publishers {
            if let Err(err) = connect_to_publisher(&mut subscription, name, &publisher, topic) {
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

fn connect_to_publisher(
    subscriber: &mut Subscriber,
    caller_id: &str,
    publisher: &str,
    topic: &str,
) -> Result<()> {
    let (protocol, hostname, port) = request_topic(publisher, caller_id, topic)?;
    if protocol != "TCPROS" {
        bail!(
            "Publisher responded with a non-TCPROS protocol: {}",
            protocol
        )
    }
    subscriber
        .connect_to((hostname.as_str(), port as u16))
        .map_err(|err| ErrorKind::Io(err).into())
}

fn request_topic(
    publisher_uri: &str,
    caller_id: &str,
    topic: &str,
) -> error::rosxmlrpc::Result<(String, String, i32)> {
    let (_code, _message, protocols): (i32, String, (String, String, i32)) = xml_rpc::Client::new()
        .unwrap()
        .call(
            &publisher_uri.parse().unwrap(),
            "requestTopic",
            &(caller_id, topic, [["TCPROS"]]),
        )
        .unwrap()
        .unwrap();
    Ok(protocols)
}

#[allow(dead_code)]
pub struct BusStats {
    pub publish: Vec<PublishStats>,
    pub subscribe: Vec<SubscribeStats>,
    pub service: ServiceStats,
}

#[allow(dead_code)]
pub struct PublishStats {
    pub name: String,
    pub data_sent: String,
    pub connection_data: PublishConnectionData,
}

#[allow(dead_code)]
pub struct PublishConnectionData {
    pub connection_id: String,
    pub bytes_sent: i32,
    pub number_sent: i32,
    pub connected: bool,
}

#[allow(dead_code)]
pub struct SubscribeStats {
    pub name: String,
    pub connection_data: SubscribeConnectionData,
}

#[allow(dead_code)]
pub struct SubscribeConnectionData {
    pub connection_id: String,
    pub bytes_received: i32,
    pub drop_estimate: i32,
    pub connected: bool,
}

#[allow(dead_code)]
pub struct ServiceStats {
    pub number_of_requests: i32,
    pub bytes_received: i32,
    pub bytes_sent: i32,
}

#[allow(dead_code)]
pub struct BusInfo {
    pub connection_id: String,
    pub destination_id: String,
    pub direction: String,
    pub transport: String,
    pub topic: String,
    pub connected: bool,
}

static FAILED_TO_LOCK: &'static str = "Failed to acquire lock";
