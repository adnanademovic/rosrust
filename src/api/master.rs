extern crate rustc_serialize;

use rosxmlrpc;
use std;
use self::rustc_serialize::{Decodable, Decoder};

pub struct Master {
    client: rosxmlrpc::Client,
    client_id: String,
    caller_api: String,
}

impl Master {
    pub fn new(client_id: &str, caller_api: &str) -> Master {
        let master_uri = std::env::var("ROS_MASTER_URI")
            .unwrap_or("http://localhost:11311/".to_owned());
        Master {
            client: rosxmlrpc::Client::new(&master_uri),
            client_id: client_id.to_owned(),
            caller_api: caller_api.to_owned(),
        }
    }

    fn remove_wrap<T>(data: MasterResult<ResponseData<T>>) -> MasterResult<T> {
        data.map(|d| d.0)
    }

    fn request<T: Decodable>(&self, function_name: &str, parameters: &[&str]) -> MasterResult<T> {
        Master::remove_wrap(self.client
            .request(function_name, parameters))
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

    pub fn get_topic_types(&self) -> MasterResult<Vec<(String, String)>> {
        self.request("getTopicTypes", &[self.client_id.as_str()])
    }

    pub fn get_system_state(&self) -> MasterResult<Vec<(String, Vec<String>)>> {
        self.request("getSystemState", &[self.client_id.as_str()])
    }

    pub fn get_uri(&self) -> MasterResult<String> {
        self.request("getUri", &[self.client_id.as_str()])
    }

    pub fn lookup_service(&self, service: &str) -> MasterResult<String> {
        self.request("lookupService", &[self.client_id.as_str(), service])
    }
}

pub type Error = rosxmlrpc::client::Error;

pub type MasterResult<T> = Result<T, Error>;

#[derive(Debug)]
pub struct ResponseData<T>(T);

impl<T: Decodable> Decodable for ResponseData<T> {
    fn decode<D: Decoder>(d: &mut D) -> Result<ResponseData<T>, D::Error> {
        d.read_struct("ResponseData", 3, |d| {
            let code = try!(d.read_struct_field("status_code", 0, |d| d.read_i32()));
            let message = try!(d.read_struct_field("status_message", 1, |d| d.read_str()));
            match code {
                0 | -1 => Err(d.error(&message)),
                1 => Ok(ResponseData(try!(d.read_struct_field("data", 2, |d| T::decode(d))))),
                _ => Err(d.error("Invalid response code returned by ROS")),
            }
        })
    }
}
