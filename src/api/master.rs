use rosxmlrpc;
use error::rosxmlrpc::{Error as ReError, ErrorKind as ReErrorKind};
use error::rosxmlrpc::serde::ErrorKind as SeErrorKind;
use super::error::master::{Result, Error, ErrorKind};
use rustc_serialize::{Decodable, Decoder, Encodable};
use super::value::Topic;

pub struct Master {
    client: rosxmlrpc::Client,
    client_id: String,
    caller_api: String,
}

macro_rules! request {
    ($s:expr; $name:ident; $($item:expr),*)=> ({
        let mut request = rosxmlrpc::client::Request::new(stringify!($name));
        request.add(&$s.client_id).map_err(|v| ReError::from(v))?;
        $(
            request.add(&$item).map_err(|v| ReError::from(v))?;
        )*
        let data : ResponseData<_> = $s.client.request(request)?;
        data.0.map_err(to_api_error)
    })
}

macro_rules! request_tree {
    ($s:expr; $name:ident; $($item:expr),*)=> ({
        let mut request = rosxmlrpc::client::Request::new(stringify!($name));
        request.add(&$s.client_id).map_err(|v| ReError::from(v))?;
        $(
            request.add(&$item).map_err(|v| ReError::from(v))?;
        )*
        $s.client.request_tree(request)
            .map_err(|v|v.into())
            .and_then(Master::remove_tree_wrap)
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

    fn remove_tree_wrap(data: rosxmlrpc::XmlRpcValue) -> Result<rosxmlrpc::XmlRpcValue> {
        let values = match data {
            rosxmlrpc::XmlRpcValue::Array(values) => values,
            _ => {
                bail!(ErrorKind::XmlRpc(ReErrorKind::Serde(SeErrorKind::MismatchedDataFormat(
                    "while handling request".into()))))
            }
        };
        if values.len() != 3 {
            bail!(ErrorKind::XmlRpc(ReErrorKind::Serde(SeErrorKind::MismatchedDataFormat("while \
                                                                                          handling \
                                                                                          request"
                .into()))))
        }
        let mut values = values.into_iter();
        let code = match values.next() {
            Some(rosxmlrpc::XmlRpcValue::Int(v)) => v,
            _ => {
                bail!(ErrorKind::XmlRpc(ReErrorKind::Serde(SeErrorKind::MismatchedDataFormat(
                    "while handling request".into()))))
            }
        };
        let message = match values.next() {
            Some(rosxmlrpc::XmlRpcValue::String(v)) => v,
            _ => {
                bail!(ErrorKind::XmlRpc(ReErrorKind::Serde(SeErrorKind::MismatchedDataFormat(
                    "while handling request".into()))))
            }
        };
        let value = match values.next() {
            Some(v) => v,
            _ => {
                bail!(ErrorKind::XmlRpc(ReErrorKind::Serde(SeErrorKind::MismatchedDataFormat(
                    "while handling request".into()))))
            }
        };
        if code != 1 {
            bail!(ErrorKind::XmlRpc(ReErrorKind::Serde(SeErrorKind::Msg(message))));
        }
        Ok(value)
    }

    pub fn register_service(&self, service: &str, service_api: &str) -> Result<i32> {
        request!(self; registerService; service, service_api, self.caller_api)
    }

    pub fn unregister_service(&self, service: &str, service_api: &str) -> Result<i32> {
        request!(self; unregisterService; service, service_api)
    }

    pub fn register_subscriber(&self, topic: &str, topic_type: &str) -> Result<Vec<String>> {
        request!(self; registerSubscriber; topic, topic_type, self.caller_api)
    }

    pub fn unregister_subscriber(&self, topic: &str) -> Result<i32> {
        request!(self; registerSubscriber; topic, self.caller_api)
    }

    pub fn register_publisher(&self, topic: &str, topic_type: &str) -> Result<Vec<String>> {
        request!(self; registerPublisher; topic, topic_type, self.caller_api)
    }

    pub fn unregister_publisher(&self, topic: &str) -> Result<i32> {
        request!(self; unregisterPublisher; topic, self.caller_api)
    }

    #[allow(dead_code)]
    pub fn lookup_node(&self, node_name: &str) -> Result<String> {
        request!(self; lookupNode; node_name)
    }

    #[allow(dead_code)]
    pub fn get_published_topics(&self, subgraph: &str) -> Result<Vec<(String, String)>> {
        request!(self; getPublishedTopics; subgraph)
    }

    pub fn get_topic_types(&self) -> Result<Vec<Topic>> {
        request!(self; getTopicTypes;)
    }

    pub fn get_system_state(&self) -> Result<SystemState> {
        request!(self; getSystemState;)
    }

    #[allow(dead_code)]
    pub fn get_uri(&self) -> Result<String> {
        request!(self; getUri;)
    }

    pub fn lookup_service(&self, service: &str) -> Result<String> {
        request!(self; lookupService; service)
    }

    pub fn delete_param(&self, key: &str) -> Result<i32> {
        request!(self; deleteParam; key)
    }

    pub fn set_param<T: Encodable>(&self, key: &str, value: &T) -> Result<i32> {
        request!(self; setParam; key, value)
    }

    pub fn get_param<T: Decodable>(&self, key: &str) -> Result<T> {
        request!(self; getParam; key)
    }

    pub fn get_param_any(&self, key: &str) -> Result<rosxmlrpc::XmlRpcValue> {
        request_tree!(self; getParam; key)
    }

    pub fn search_param(&self, key: &str) -> Result<String> {
        request!(self; searchParam; key)
    }

    #[allow(dead_code)]
    pub fn subscribe_param<T: Decodable>(&self, key: &str) -> Result<T> {
        request!(self; subscribeParam; self.caller_api, key)
    }

    #[allow(dead_code)]
    pub fn subscribe_param_any(&self, key: &str) -> Result<rosxmlrpc::XmlRpcValue> {
        request_tree!(self; subscribeParam; self.caller_api, key)
    }

    #[allow(dead_code)]
    pub fn unsubscribe_param(&self, key: &str) -> Result<i32> {
        request!(self; unsubscribeParam; self.caller_api, key)
    }

    pub fn has_param(&self, key: &str) -> Result<bool> {
        request!(self; hasParam; key)
    }

    pub fn get_param_names(&self) -> Result<Vec<String>> {
        request!(self; getParamNames;)
    }
}

fn to_api_error(v: (bool, String)) -> Error {
    use super::error::api::ErrorKind as ApiErrorKind;
    match v.0 {
            false => ErrorKind::Api(ApiErrorKind::SystemFail(v.1)),
            true => ErrorKind::Api(ApiErrorKind::BadData(v.1)),
        }
        .into()
}

#[derive(Debug)]
struct ResponseData<T>(::std::result::Result<T, (bool, String)>);

impl<T: Decodable> Decodable for ResponseData<T> {
    fn decode<D: Decoder>(d: &mut D) -> ::std::result::Result<ResponseData<T>, D::Error> {
        d.read_struct("ResponseData", 3, |d| {
            let code = d.read_struct_field("status_code", 0, |d| d.read_i32())?;
            let message = d.read_struct_field("status_message", 1, |d| d.read_str())?;
            match code {
                0 | -1 => Ok(ResponseData(Err((code != 0, message)))),
                1 => Ok(ResponseData(Ok(d.read_struct_field("data", 2, |d| T::decode(d))?))),
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
