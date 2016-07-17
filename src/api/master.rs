use rosxmlrpc;
use std;

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

    pub fn register_service(&self,
                            service: &str,
                            service_api: &str)
                            -> rosxmlrpc::client::ClientResult {
        self.client
            .request("registerService",
                     &[self.client_id.as_str(), service, service_api, self.caller_api.as_str()])
    }

    pub fn unregister_service(&self,
                              service: &str,
                              service_api: &str)
                              -> rosxmlrpc::client::ClientResult {
        self.client
            .request("unregisterService",
                     &[self.client_id.as_str(), service, service_api])
    }

    pub fn register_subscriber(&self,
                               topic: &str,
                               topic_type: &str)
                               -> rosxmlrpc::client::ClientResult {
        self.client
            .request("registerSubscriber",
                     &[self.client_id.as_str(), topic, topic_type, self.caller_api.as_str()])
    }

    pub fn unregister_subscriber(&self, topic: &str) -> rosxmlrpc::client::ClientResult {
        self.client
            .request("unregisterSubscriber",
                     &[self.client_id.as_str(), topic, self.caller_api.as_str()])
    }

    pub fn register_publisher(&self,
                              topic: &str,
                              topic_type: &str)
                              -> rosxmlrpc::client::ClientResult {
        self.client
            .request("registerPublisher",
                     &[self.client_id.as_str(), topic, topic_type, self.caller_api.as_str()])
    }

    pub fn unregister_publisher(&self, topic: &str) -> rosxmlrpc::client::ClientResult {
        self.client
            .request("unregisterPublisher",
                     &[self.client_id.as_str(), topic, self.caller_api.as_str()])
    }

    pub fn lookup_node(&self, node_name: &str) -> rosxmlrpc::client::ClientResult {
        self.client
            .request("lookupNode", &[self.client_id.as_str(), node_name])
    }

    pub fn get_published_topics(&self, subgraph: &str) -> rosxmlrpc::client::ClientResult {
        self.client
            .request("getPublishedTopics", &[self.client_id.as_str(), subgraph])
    }

    pub fn get_topic_types(&self) -> rosxmlrpc::client::ClientResult {
        self.client
            .request("getTopicTypes", &[self.client_id.as_str()])
    }

    pub fn get_system_state(&self) -> rosxmlrpc::client::ClientResult {
        self.client
            .request("getSystemState", &[self.client_id.as_str()])
    }

    pub fn get_uri(&self) -> rosxmlrpc::client::ClientResult {
        self.client
            .request("getUri", &[self.client_id.as_str()])
    }

    pub fn lookup_service(&self, service: &str) -> rosxmlrpc::client::ClientResult {
        self.client
            .request("lookupService", &[self.client_id.as_str(), service])
    }
}

#[derive(Debug)]
pub enum Error {
    XmlRpc(rosxmlrpc::client::Error),
    Ros(String),
    Format,
}

type MasterResult<T> = Result<T, Error>;

impl From<rosxmlrpc::client::Error> for Error {
    fn from(err: rosxmlrpc::client::Error) -> Error {
        Error::XmlRpc(err)
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match *self {
            Error::XmlRpc(ref err) => write!(f, "XML RPC error: {}", err),
            Error::Ros(ref err) => write!(f, "ROS core error: {}", err),
            Error::Format => write!(f, "Bad response format"),
        }
    }
}

impl std::error::Error for Error {
    fn description(&self) -> &str {
        match *self {
            Error::XmlRpc(ref err) => err.description(),
            Error::Ros(ref err) => &err,
            Error::Format => "Bad response format received, expected [int, string, *]",
        }
    }

    fn cause(&self) -> Option<&std::error::Error> {
        match *self {
            Error::XmlRpc(ref err) => Some(err),
            Error::Ros(..) => None,
            Error::Format => None,
        }
    }
}
