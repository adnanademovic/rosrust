use regex::Regex;
use rosxmlrpc;
use rosxmlrpc::error::{Error as ReError, ErrorKind as ReErrorKind};
use rosxmlrpc::serde::{Error as SeError, ErrorKind as SeErrorKind};
use rustc_serialize::{Decodable, Decoder, Encodable};
use std;
use super::value::Topic;

pub struct Master {
    client: rosxmlrpc::Client,
    client_id: String,
    caller_api: String,
}

macro_rules! request {
    ($s:expr; $name:ident; $($item:expr),*)=> ({
        let mut request = rosxmlrpc::client::Request::new(stringify!($name));
        request.add(&$s.client_id)?;
        $(
            request.add(&$item)?;
        )*
        let data : ResponseData<_> = $s.client.request(request)?;
        Ok(data.0)
    })
}

macro_rules! request_tree {
    ($s:expr; $name:ident; $($item:expr),*)=> ({
        let mut request = rosxmlrpc::client::Request::new(stringify!($name));
        request.add(&$s.client_id)?;
        $(
            request.add(&$item)?;
        )*
        Master::remove_tree_wrap($s.client.request_tree(request))
    })
}

impl Master {
    pub fn new(master_uri: &str, client_id: &str, caller_api: &str) -> Master {
        Master {
            client: rosxmlrpc::Client::new(&master_uri),
            client_id: client_id.to_owned(),
            caller_api: caller_api.to_owned(),
        }
    }

    fn remove_tree_wrap(data: Result<rosxmlrpc::XmlRpcValue, ReError>)
                        -> MasterResult<rosxmlrpc::XmlRpcValue> {
        let values = match data? {
            rosxmlrpc::XmlRpcValue::Array(values) => values,
            _ => return Err(MasterError::XmlRpc(
                ReErrorKind::Serde(SeErrorKind::MismatchedDataFormat(
                    "while handling request".into())).into())),
        };
        if values.len() != 3 {
            return Err(MasterError::XmlRpc(
                ReErrorKind::Serde(SeErrorKind::MismatchedDataFormat(
                    "while handling request".into())).into()));
        }
        let mut values = values.into_iter();
        let code = match values.next() {
            Some(rosxmlrpc::XmlRpcValue::Int(v)) => v,
            _ => return Err(MasterError::XmlRpc(
                ReErrorKind::Serde(SeErrorKind::MismatchedDataFormat(
                    "while handling request".into())).into())),
        };
        let message = match values.next() {
            Some(rosxmlrpc::XmlRpcValue::String(v)) => v,
            _ => return Err(MasterError::XmlRpc(
                ReErrorKind::Serde(SeErrorKind::MismatchedDataFormat(
                    "while handling request".into())).into())),
        };
        let value = values.next()
            .ok_or(MasterError::XmlRpc(
                ReErrorKind::Serde(SeErrorKind::MismatchedDataFormat(
                    "while handling request".into())).into()))?;
        match code {
            0 | -1 => Err(SeError::from(message))?,
            1 => Ok(value),
            v => {
                warn!("ROS Master returned '{}' response code (only -1, 0, 1 legal)",
                      v);
                Err(SeError::from("Invalid response code returned by ROS"))?
            }
        }
    }

    pub fn register_service(&self, service: &str, service_api: &str) -> MasterResult<i32> {
        request!(self; registerService; service, service_api, self.caller_api)
    }

    pub fn unregister_service(&self, service: &str, service_api: &str) -> MasterResult<i32> {
        request!(self; unregisterService; service, service_api)
    }

    pub fn register_subscriber(&self, topic: &str, topic_type: &str) -> MasterResult<Vec<String>> {
        request!(self; registerSubscriber; topic, topic_type, self.caller_api)
    }

    pub fn unregister_subscriber(&self, topic: &str) -> MasterResult<i32> {
        request!(self; registerSubscriber; topic, self.caller_api)
    }

    pub fn register_publisher(&self, topic: &str, topic_type: &str) -> MasterResult<Vec<String>> {
        request!(self; registerPublisher; topic, topic_type, self.caller_api)
    }

    pub fn unregister_publisher(&self, topic: &str) -> MasterResult<i32> {
        request!(self; unregisterPublisher; topic, self.caller_api)
    }

    #[allow(dead_code)]
    pub fn lookup_node(&self, node_name: &str) -> MasterResult<String> {
        request!(self; lookupNode; node_name)
    }

    #[allow(dead_code)]
    pub fn get_published_topics(&self, subgraph: &str) -> MasterResult<Vec<(String, String)>> {
        request!(self; getPublishedTopics; subgraph)
    }

    pub fn get_topic_types(&self) -> MasterResult<Vec<Topic>> {
        request!(self; getTopicTypes;)
    }

    pub fn get_system_state(&self) -> MasterResult<SystemState> {
        request!(self; getSystemState;)
    }

    #[allow(dead_code)]
    pub fn get_uri(&self) -> MasterResult<String> {
        request!(self; getUri;)
    }

    pub fn lookup_service(&self, service: &str) -> MasterResult<String> {
        request!(self; lookupService; service)
    }

    pub fn delete_param(&self, key: &str) -> MasterResult<i32> {
        request!(self; deleteParam; key)
    }

    pub fn set_param<T: Encodable>(&self, key: &str, value: &T) -> MasterResult<i32> {
        request!(self; setParam; key, value)
    }

    pub fn get_param<T: Decodable>(&self, key: &str) -> MasterResult<T> {
        request!(self; getParam; key)
    }

    pub fn get_param_any(&self, key: &str) -> MasterResult<rosxmlrpc::XmlRpcValue> {
        request_tree!(self; getParam; key)
    }

    pub fn search_param(&self, key: &str) -> MasterResult<String> {
        request!(self; searchParam; key)
    }

    #[allow(dead_code)]
    pub fn subscribe_param<T: Decodable>(&self, key: &str) -> MasterResult<T> {
        request!(self; subscribeParam; self.caller_api, key)
    }

    #[allow(dead_code)]
    pub fn subscribe_param_any(&self, key: &str) -> MasterResult<rosxmlrpc::XmlRpcValue> {
        request_tree!(self; subscribeParam; self.caller_api, key)
    }

    #[allow(dead_code)]
    pub fn unsubscribe_param(&self, key: &str) -> MasterResult<i32> {
        request!(self; unsubscribeParam; self.caller_api, key)
    }

    pub fn has_param(&self, key: &str) -> MasterResult<bool> {
        request!(self; hasParam; key)
    }

    pub fn get_param_names(&self) -> MasterResult<Vec<String>> {
        request!(self; getParamNames;)
    }
}

pub type MasterResult<T> = Result<T, MasterError>;

#[derive(Debug)]
struct ResponseData<T>(T);

impl<T: Decodable> Decodable for ResponseData<T> {
    fn decode<D: Decoder>(d: &mut D) -> Result<ResponseData<T>, D::Error> {
        d.read_struct("ResponseData", 3, |d| {
            let code = d.read_struct_field("status_code", 0, |d| d.read_i32())?;
            let message = d.read_struct_field("status_message", 1, |d| d.read_str())?;
            match code {
                0 | -1 => Err(d.error(&format!("ROS MASTER ERROR CODE {}: {}", -code, message))),
                1 => Ok(ResponseData(d.read_struct_field("data", 2, |d| T::decode(d))?)),
                _ => Err(d.error("Invalid response code returned by ROS")),
            }
        })
    }
}

#[derive(Debug,RustcDecodable)]
pub struct TopicData {
    pub name: String,
    pub connections: Vec<String>,
}

#[derive(Debug,RustcDecodable)]
pub struct SystemState {
    pub publishers: Vec<TopicData>,
    pub subscribers: Vec<TopicData>,
    pub services: Vec<TopicData>,
}

#[derive(Debug)]
pub enum FailureType {
    Failure,
    Error,
}

#[derive(Debug)]
pub enum MasterError {
    XmlRpc(ReError),
    ApiError(FailureType, String),
}

impl From<ReError> for MasterError {
    fn from(err: ReError) -> MasterError {
        if let ReError(ReErrorKind::Serde(SeErrorKind::Msg(ref v)), _) = err {
            lazy_static!{
                static ref RE: Regex = Regex::new("^ROS MASTER ERROR CODE ([01]): (.*)$").unwrap();
            }
            if let Some(cap) = RE.captures(&v) {
                let failure_type = if cap.at(1) == Some("0") {
                    FailureType::Failure
                } else {
                    FailureType::Error
                };
                let message = String::from(cap.at(2).unwrap_or(""));
                return MasterError::ApiError(failure_type, message);
            }
        }
        MasterError::XmlRpc(err)
    }
}

impl From<SeError> for MasterError {
    fn from(err: SeError) -> MasterError {
        MasterError::XmlRpc(err.into())
    }
}

impl std::fmt::Display for FailureType {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f,
               "{}",
               match *self {
                   FailureType::Error => "ERROR",
                   FailureType::Failure => "FAILURE",
               })
    }
}

impl std::fmt::Display for MasterError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        use std::error::Error as ErrorTrait;
        match *self {
            MasterError::XmlRpc(ref err) => write!(f, "XML RPC error: {}", err),
            MasterError::ApiError(..) => write!(f, "Error with ROS API: {}", self.description()),
        }
    }
}

impl std::error::Error for MasterError {
    fn description(&self) -> &str {
        match *self {
            MasterError::XmlRpc(ref err) => err.description(),
            MasterError::ApiError(.., ref msg) => msg,
        }
    }

    fn cause(&self) -> Option<&std::error::Error> {
        match *self {
            MasterError::XmlRpc(ref err) => Some(err),
            MasterError::ApiError(..) => None,
        }
    }
}
