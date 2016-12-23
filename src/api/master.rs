use rosxmlrpc;
use rosxmlrpc::error::Error;
use rosxmlrpc::serde::decoder::Error as DecoderError;
use rustc_serialize::{Decodable, Decoder, Encodable};
use super::value::Topic;

pub struct Master {
    client: rosxmlrpc::Client,
    client_id: String,
    caller_api: String,
}

const MISMATCHED_FORMAT: Error = Error::Deserialization(DecoderError::MismatchedDataFormat);

impl Master {
    pub fn new(master_uri: &str, client_id: &str, caller_api: &str) -> Master {
        Master {
            client: rosxmlrpc::Client::new(&master_uri),
            client_id: client_id.to_owned(),
            caller_api: caller_api.to_owned(),
        }
    }

    fn remove_wrap<T>(data: MasterResult<ResponseData<T>>) -> MasterResult<T> {
        data.map(|d| d.0)
    }

    fn remove_tree_wrap(data: MasterResult<rosxmlrpc::XmlRpcValue>)
                        -> MasterResult<rosxmlrpc::XmlRpcValue> {
        let values = match data? {
            rosxmlrpc::XmlRpcValue::Array(values) => values,
            _ => return Err(MISMATCHED_FORMAT),
        };
        if values.len() != 3 {
            return Err(MISMATCHED_FORMAT);
        }
        let mut values = values.into_iter();
        let code = match values.next() {
            Some(rosxmlrpc::XmlRpcValue::Int(v)) => v,
            _ => return Err(MISMATCHED_FORMAT),
        };
        let message = match values.next() {
            Some(rosxmlrpc::XmlRpcValue::String(v)) => v,
            _ => return Err(MISMATCHED_FORMAT),
        };
        let value = values.next()
            .ok_or(rosxmlrpc::serde::decoder::Error::MismatchedDataFormat)?;
        match code {
            0 | -1 => Err(rosxmlrpc::serde::decoder::Error::Other(message))?,
            1 => Ok(value),
            v => {
                warn!("ROS Master returned '{}' response code (only -1, 0, 1 legal)",
                      v);
                Err(rosxmlrpc::serde::decoder::Error::Other(String::from("Invalid response \
                                                                          code returned by ROS")))?
            }
        }
    }

    fn request<T: Decodable>(&self, function_name: &str, parameters: &[&str]) -> MasterResult<T> {
        let mut request = rosxmlrpc::client::Request::new(function_name);
        for parameter in parameters {
            request.add(parameter)?;
        }
        Master::remove_wrap(self.client.request(request))
    }

    pub fn register_service(&self, service: &str, service_api: &str) -> MasterResult<i32> {
        self.request("registerService",
                     &[self.client_id.as_str(), service, service_api, self.caller_api.as_str()])
    }

    pub fn unregister_service(&self, service: &str, service_api: &str) -> MasterResult<i32> {
        self.request("unregisterService",
                     &[self.client_id.as_str(), service, service_api])
    }

    pub fn register_subscriber(&self, topic: &str, topic_type: &str) -> MasterResult<Vec<String>> {
        self.request("registerSubscriber",
                     &[self.client_id.as_str(), topic, topic_type, self.caller_api.as_str()])
    }

    pub fn unregister_subscriber(&self, topic: &str) -> MasterResult<i32> {
        self.request("unregisterSubscriber",
                     &[self.client_id.as_str(), topic, self.caller_api.as_str()])
    }

    pub fn register_publisher(&self, topic: &str, topic_type: &str) -> MasterResult<Vec<String>> {
        self.request("registerPublisher",
                     &[self.client_id.as_str(), topic, topic_type, self.caller_api.as_str()])
    }

    pub fn unregister_publisher(&self, topic: &str) -> MasterResult<i32> {
        self.request("unregisterPublisher",
                     &[self.client_id.as_str(), topic, self.caller_api.as_str()])
    }

    pub fn lookup_node(&self, node_name: &str) -> MasterResult<String> {
        self.request("lookupNode", &[self.client_id.as_str(), node_name])
    }

    pub fn get_published_topics(&self, subgraph: &str) -> MasterResult<Vec<(String, String)>> {
        self.request("getPublishedTopics", &[self.client_id.as_str(), subgraph])
    }

    pub fn get_topic_types(&self) -> MasterResult<Vec<Topic>> {
        self.request("getTopicTypes", &[self.client_id.as_str()])
    }

    pub fn get_system_state(&self) -> MasterResult<SystemState> {
        self.request("getSystemState", &[self.client_id.as_str()])
    }

    pub fn get_uri(&self) -> MasterResult<String> {
        self.request("getUri", &[self.client_id.as_str()])
    }

    pub fn lookup_service(&self, service: &str) -> MasterResult<String> {
        self.request("lookupService", &[self.client_id.as_str(), service])
    }

    pub fn delete_param(&self, key: &str) -> MasterResult<i32> {
        self.request("deleteParam", &[self.client_id.as_str(), key])
    }

    pub fn set_param<T: Encodable>(&self, key: &str, value: &T) -> MasterResult<i32> {
        let mut request = rosxmlrpc::client::Request::new("setParam");
        request.add(&self.client_id)?;
        request.add(&key)?;
        request.add(value)?;
        Master::remove_wrap(self.client.request(request))
    }

    pub fn get_param<T: Decodable>(&self, key: &str) -> MasterResult<T> {
        self.request("getParam", &[self.client_id.as_str(), key])
    }

    pub fn get_param_any(&self, key: &str) -> MasterResult<rosxmlrpc::XmlRpcValue> {
        let mut request = rosxmlrpc::client::Request::new("getParam");
        request.add(&self.client_id)?;
        request.add(&key)?;
        Master::remove_tree_wrap(self.client.request_tree(request))
    }

    pub fn search_param(&self, key: &str) -> MasterResult<String> {
        self.request("searchParam", &[self.client_id.as_str(), key])
    }

    pub fn subscribe_param<T: Decodable>(&self, key: &str) -> MasterResult<T> {
        self.request("subscribeParam",
                     &[self.client_id.as_str(), self.caller_api.as_str(), key])
    }

    pub fn subscribe_param_any(&self, key: &str) -> MasterResult<rosxmlrpc::XmlRpcValue> {
        let mut request = rosxmlrpc::client::Request::new("subscribeParam");
        request.add(&self.client_id)?;
        request.add(&self.caller_api)?;
        request.add(&key)?;
        Master::remove_tree_wrap(self.client.request_tree(request))
    }

    pub fn unsubscribe_param(&self, key: &str) -> MasterResult<i32> {
        self.request("unsubscribeParam",
                     &[self.client_id.as_str(), self.caller_api.as_str(), key])
    }

    pub fn has_param(&self, key: &str) -> MasterResult<bool> {
        self.request("hasParam", &[self.client_id.as_str(), key])
    }

    pub fn get_param_names(&self) -> MasterResult<Vec<String>> {
        self.request("getParamNames", &[self.client_id.as_str()])
    }
}

pub type MasterResult<T> = Result<T, Error>;

#[derive(Debug)]
struct ResponseData<T>(T);

impl<T: Decodable> Decodable for ResponseData<T> {
    fn decode<D: Decoder>(d: &mut D) -> Result<ResponseData<T>, D::Error> {
        d.read_struct("ResponseData", 3, |d| {
            let code = d.read_struct_field("status_code", 0, |d| d.read_i32())?;
            let message = d.read_struct_field("status_message", 1, |d| d.read_str())?;
            match code {
                0 | -1 => Err(d.error(&message)),
                1 => Ok(ResponseData(d.read_struct_field("data", 2, |d| T::decode(d))?)),
                _ => Err(d.error("Invalid response code returned by ROS")),
            }
        })
    }
}

#[derive(RustcDecodable)]
pub struct TopicData {
    pub name: String,
    pub connections: Vec<String>,
}

#[derive(RustcDecodable)]
pub struct SystemState {
    pub publishers: Vec<TopicData>,
    pub subscribers: Vec<TopicData>,
    pub services: Vec<TopicData>,
}
