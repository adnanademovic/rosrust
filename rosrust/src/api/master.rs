use serde::{Deserialize, Serialize};
use super::super::rosxmlrpc::{self, Response as Result};
use xml_rpc;

pub struct Master {
    client: rosxmlrpc::Client,
    client_id: String,
    caller_api: String,
}

macro_rules! request {
    ($s:expr; $name:ident; $($item:expr),*)=> ({
        $s.client.request(stringify!($name),&(&$s.client_id,
            $(
                $item,
            )*
            ))
    })
}

macro_rules! request_tree {
    ($s:expr; $name:ident; $($item:expr),*)=> ({
        $s.client.request_tree(stringify!($name),&(&$s.client_id,
            $(
                $item,
            )*
            ))
    })
}

impl Master {
    pub fn new(master_uri: &str, client_id: &str, caller_api: &str) -> Master {
        Master {
            client: rosxmlrpc::Client::new(master_uri).unwrap(),
            client_id: client_id.to_owned(),
            caller_api: caller_api.to_owned(),
        }
    }

    pub fn register_service(&self, service: &str, service_api: &str) -> Result<i32> {
        request!(self; registerService; service, service_api, &self.caller_api)
    }

    pub fn unregister_service(&self, service: &str, service_api: &str) -> Result<i32> {
        request!(self; unregisterService; service, service_api)
    }

    pub fn register_subscriber(&self, topic: &str, topic_type: &str) -> Result<Vec<String>> {
        request!(self; registerSubscriber; topic, topic_type, &self.caller_api)
    }

    pub fn unregister_subscriber(&self, topic: &str) -> Result<i32> {
        request!(self; unregisterSubscriber; topic, &self.caller_api)
    }

    pub fn register_publisher(&self, topic: &str, topic_type: &str) -> Result<Vec<String>> {
        request!(self; registerPublisher; topic, topic_type, &self.caller_api)
    }

    pub fn unregister_publisher(&self, topic: &str) -> Result<i32> {
        request!(self; unregisterPublisher; topic, &self.caller_api)
    }

    #[allow(dead_code)]
    pub fn lookup_node(&self, node_name: &str) -> Result<String> {
        request!(self; lookupNode; node_name)
    }

    #[allow(dead_code)]
    pub fn get_published_topics(&self, subgraph: &str) -> Result<Vec<(String, String)>> {
        request!(self; getPublishedTopics; subgraph)
    }

    pub fn get_topic_types(&self) -> Result<Vec<TopicTuple>> {
        request!(self; getTopicTypes;)
    }

    pub fn get_system_state(&self) -> Result<SystemStateTuple> {
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

    pub fn set_param<T: Serialize>(&self, key: &str, value: &T) -> Result<i32> {
        request!(self; setParam; key, value)
    }

    pub fn set_param_any(&self, key: &str, value: xml_rpc::Value) -> Result<()> {
        self.client
            .request_tree_with_tree(
                "setParam",
                vec![
                    xml_rpc::Value::String(self.client_id.clone()),
                    xml_rpc::Value::String(key.into()),
                    value,
                ],
            )
            .and(Ok(()))
    }

    pub fn get_param<'a, T: Deserialize<'a>>(&self, key: &str) -> Result<T> {
        request!(self; getParam; key)
    }

    pub fn get_param_any(&self, key: &str) -> Result<xml_rpc::Value> {
        request_tree!(self; getParam; key)
    }

    pub fn search_param(&self, key: &str) -> Result<String> {
        request!(self; searchParam; key)
    }

    #[allow(dead_code)]
    pub fn subscribe_param<'a, T: Deserialize<'a>>(&self, key: &str) -> Result<T> {
        request!(self; subscribeParam; &self.caller_api, key)
    }

    #[allow(dead_code)]
    pub fn subscribe_param_any(&self, key: &str) -> Result<xml_rpc::Value> {
        request_tree!(self; subscribeParam; &self.caller_api, key)
    }

    #[allow(dead_code)]
    pub fn unsubscribe_param(&self, key: &str) -> Result<i32> {
        request!(self; unsubscribeParam; &self.caller_api, key)
    }

    pub fn has_param(&self, key: &str) -> Result<bool> {
        request!(self; hasParam; key)
    }

    pub fn get_param_names(&self) -> Result<Vec<String>> {
        request!(self; getParamNames;)
    }
}

#[derive(Debug)]
pub struct TopicData {
    pub name: String,
    pub connections: Vec<String>,
}

#[derive(Debug)]
pub struct SystemState {
    pub publishers: Vec<TopicData>,
    pub subscribers: Vec<TopicData>,
    pub services: Vec<TopicData>,
}

#[derive(Debug, Deserialize)]
pub struct TopicDataTuple(String, Vec<String>);
#[derive(Debug, Deserialize)]
pub struct SystemStateTuple(
    Vec<TopicDataTuple>,
    Vec<TopicDataTuple>,
    Vec<TopicDataTuple>,
);

impl Into<SystemState> for SystemStateTuple {
    fn into(self) -> SystemState {
        SystemState {
            publishers: self.0.into_iter().map(Into::into).collect(),
            subscribers: self.1.into_iter().map(Into::into).collect(),
            services: self.2.into_iter().map(Into::into).collect(),
        }
    }
}

impl Into<TopicData> for TopicDataTuple {
    fn into(self) -> TopicData {
        TopicData {
            name: self.0,
            connections: self.1,
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct TopicTuple(String, String);

impl Into<Topic> for TopicTuple {
    fn into(self) -> Topic {
        Topic {
            name: self.0,
            datatype: self.1,
        }
    }
}

pub struct Topic {
    pub name: String,
    pub datatype: String,
}
