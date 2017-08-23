use std;
use std::sync::Mutex;
use super::error::master::{Result, Error, ErrorKind};
use serde::{Deserialize, Serialize};
use super::value::Topic;
use xml_rpc;

pub struct Master {
    client: Mutex<xml_rpc::Client>,
    client_id: String,
    caller_api: String,
    master_uri: String,
}

const ERROR_CODE: i32 = -1;
const FAILURE_CODE: i32 = 0;
const SUCCESS_CODE: i32 = 1;

fn parse_response(params: xml_rpc::Params) -> std::result::Result<xml_rpc::Value, xml_rpc::Fault> {
    let mut param_iter = params.into_iter();
    let code = param_iter.next().ok_or_else(|| {
        xml_rpc::Fault::new(FAILURE_CODE, "Server response missing arguments.")
    })?;
    let message = param_iter.next().ok_or_else(|| {
        xml_rpc::Fault::new(FAILURE_CODE, "Server response missing arguments.")
    })?;
    let value = param_iter.next().ok_or_else(|| {
        xml_rpc::Fault::new(FAILURE_CODE, "Server response missing arguments.")
    })?;
    let code = match code {
        xml_rpc::Value::Int(v) => v,
        _ => {
            return Err(xml_rpc::Fault::new(
                FAILURE_CODE,
                "First response argument is expected to be int.",
            ))
        }
    };
    let message = match message {
        xml_rpc::Value::String(v) => v,
        _ => {
            return Err(xml_rpc::Fault::new(
                FAILURE_CODE,
                "Second response argument is expected to be string.",
            ))
        }
    };
    if code != SUCCESS_CODE {
        return Err(xml_rpc::Fault::new(code, message));
    }
    Ok(value)
}

macro_rules! request {
    ($s:expr; $name:ident; $($item:expr),*)=> ({
        let params = xml_rpc::into_params(&(&$s.client_id,
            $(
                $item,
            )*
            ))
            .map_err(xml_rpc::error::Error::from)?;
        let response = $s.client.lock().unwrap()
            .call_value(&$s.master_uri.parse().unwrap(), stringify!($name), params)?
            .map_err(to_api_error)?;
        let data = parse_response(response).map_err(to_api_error)?;
        Deserialize::deserialize(data).map_err(|v| {
        to_api_error(xml_rpc::Fault::new(
            FAILURE_CODE,
            format!("Third response argument has unexpected structure: {}", v),
        ))})
    })
}

macro_rules! request_tree {
    ($s:expr; $name:ident; $($item:expr),*)=> ({
        let params = xml_rpc::into_params(&(&$s.client_id,
            $(
                $item,
            )*
            ))
            .map_err(xml_rpc::error::Error::from)?;
        let response = $s.client.lock().unwrap()
            .call_value(&$s.master_uri.parse().unwrap(), stringify!($name), params)?
            .map_err(ErrorKind::Fault)?;
        parse_response(response).map_err(to_api_error)
    })
}

impl Master {
    pub fn new(master_uri: &str, client_id: &str, caller_api: &str) -> Master {
        Master {
            client: Mutex::new(xml_rpc::Client::new().unwrap()),
            client_id: client_id.to_owned(),
            caller_api: caller_api.to_owned(),
            master_uri: master_uri.to_owned(),
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
        request!(self; registerSubscriber; topic, &self.caller_api)
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

fn to_api_error(v: xml_rpc::Fault) -> Error {
    use super::error::api::ErrorKind as ApiErrorKind;
    match v.code {
        FAILURE_CODE => ErrorKind::Api(ApiErrorKind::SystemFail(v.message)),
        ERROR_CODE => ErrorKind::Api(ApiErrorKind::BadData(v.message)),
        x => ErrorKind::Api(ApiErrorKind::SystemFail(format!(
            "Bad error code #{} returned with message: {}",
            x,
            v.message
        ))),
    }.into()
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
pub struct SystemStateTuple(Vec<TopicDataTuple>, Vec<TopicDataTuple>, Vec<TopicDataTuple>);

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
