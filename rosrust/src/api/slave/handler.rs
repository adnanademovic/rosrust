use super::publications::PublicationsTracker;
use super::subscriptions::SubscriptionsTracker;
use crate::rosxmlrpc::{self, Response, ResponseError, Server};
use crate::tcpros::Service;
use crate::util::{kill, FAILED_TO_LOCK};
use log::{error, info};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use xml_rpc::{self, rouille, Params, Value};

pub struct SlaveHandler {
    pub subscriptions: SubscriptionsTracker,
    pub publications: PublicationsTracker,
    pub services: Arc<Mutex<HashMap<String, Service>>>,
    server: Server,
}

fn unwrap_array_case(params: Params) -> Params {
    if let Some(Value::Array(items)) = params.get(0) {
        return items.clone();
    }
    params
}

#[derive(Default)]
pub struct ParamCacheState {
    pub data: HashMap<String, Response<Value>>,
    pub subscribed: bool,
}

pub type ParamCache = Arc<Mutex<ParamCacheState>>;

impl SlaveHandler {
    pub fn new(
        master_uri: &str,
        hostname: &str,
        name: &str,
        param_cache: ParamCache,
        shutdown_signal: kill::Sender,
    ) -> SlaveHandler {
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
            let _caller_id = args
                .next()
                .ok_or_else(|| ResponseError::Client("Missing argument 'caller_id'".into()))?;
            let message = match args.next() {
                Some(Value::String(message)) => message,
                _ => return Err(ResponseError::Client("Missing argument 'message'".into())),
            };
            info!("Server is shutting down because: {}", message);
            match shutdown_signal.send() {
                Ok(()) => Ok(Value::Int(0)),
                Err(err) => {
                    error!("Shutdown error: {:?}", err);
                    Err(ResponseError::Server("Failed to shut down".into()))
                }
            }
        });

        server.register_value("getPid", "PID", |_args| {
            Ok(Value::Int(std::process::id() as i32))
        });

        let subscriptions = SubscriptionsTracker::default();
        let subs = subscriptions.clone();

        server.register_value("getSubscriptions", "List of subscriptions", move |_args| {
            Ok(Value::Array(
                subs.get_topics::<Vec<_>>()
                    .into_iter()
                    .map(|topic| {
                        Value::Array(vec![
                            Value::String(topic.name),
                            Value::String(topic.msg_type),
                        ])
                    })
                    .collect(),
            ))
        });

        let publications = PublicationsTracker::default();
        let pubs = publications.clone();

        server.register_value("getPublications", "List of publications", move |_args| {
            Ok(Value::Array(
                pubs.get_topics::<Vec<_>>()
                    .into_iter()
                    .map(|topic| {
                        Value::Array(vec![
                            Value::String(topic.name),
                            Value::String(topic.msg_type),
                        ])
                    })
                    .collect(),
            ))
        });

        server.register_value("paramUpdate", "Parameter updated", move |args| {
            let mut args = unwrap_array_case(args).into_iter();
            let _caller_id = args
                .next()
                .ok_or_else(|| ResponseError::Client("Missing argument 'caller_id'".into()))?;
            let parameter_key = match args.next() {
                Some(Value::String(parameter_key)) => parameter_key,
                _ => {
                    return Err(ResponseError::Client(
                        "Missing argument 'parameter_key'".into(),
                    ))
                }
            };
            let _parameter_value = match args.next() {
                Some(parameter_value) => parameter_value,
                _ => {
                    return Err(ResponseError::Client(
                        "Missing argument 'parameter_key'".into(),
                    ))
                }
            };
            let key = parameter_key.trim_end_matches('/');
            param_cache
                .lock()
                .expect(FAILED_TO_LOCK)
                .data
                .retain(|k, _| !k.starts_with(key) && !key.starts_with(k));
            Ok(Value::Int(0))
        });

        let name_string = String::from(name);
        let subs = subscriptions.clone();

        server.register_value("publisherUpdate", "Publishers updated", move |args| {
            let mut args = unwrap_array_case(args).into_iter();
            let _caller_id = args
                .next()
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
                    ));
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

            subs.add_publishers(&topic, &name_string, publishers.into_iter())
                .map_err(|v| {
                    ResponseError::Server(format!("Failed to handle publishers: {}", v))
                })?;
            Ok(Value::Int(0))
        });

        let hostname_string = String::from(hostname);
        let pubs = publications.clone();

        server.register_value("requestTopic", "Chosen protocol", move |args| {
            let mut args = unwrap_array_case(args).into_iter();
            let _caller_id = args
                .next()
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
                    ));
                }
                None => return Err(ResponseError::Client("Missing argument 'protocols'".into())),
            };
            let port = pubs.get_port(&topic).ok_or_else(|| {
                ResponseError::Client("Requested topic not published by node".into())
            })?;
            let ip = hostname_string.clone();
            let mut has_tcpros = false;
            for protocol in protocols {
                if let Value::Array(protocol) = protocol {
                    if let Some(Value::String(name)) = protocol.get(0) {
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
            subscriptions,
            publications,
            services: Arc::new(Mutex::new(HashMap::new())),
            server,
        }
    }

    pub fn bind(
        self,
        addr: &SocketAddr,
    ) -> rosxmlrpc::error::Result<
        xml_rpc::server::BoundServer<
            impl Fn(&rouille::Request) -> rouille::Response + Send + Sync + 'static,
        >,
    > {
        self.server.bind(addr).map_err(Into::into)
    }
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
