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
            .unwrap_or("http://localhost:11311/".to_string());
        Master {
            client: rosxmlrpc::Client::new(&master_uri),
            client_id: client_id.to_string(),
            caller_api: caller_api.to_string(),
        }
    }

    pub fn register_service(&self, service: &str, service_api: &str) -> rosxmlrpc::client::Member {
        self.client.request("registerService",
                            &[self.client_id.as_str(),
                              service,
                              service_api,
                              self.caller_api.as_str()])
    }

    pub fn unregister_service(&self,
                              service: &str,
                              service_api: &str)
                              -> rosxmlrpc::client::Member {
        self.client.request("unregisterService",
                            &[self.client_id.as_str(), service, service_api])
    }

    pub fn register_subscriber(&self, topic: &str, topic_type: &str) -> rosxmlrpc::client::Member {
        self.client.request("registerSubscriber",
                            &[self.client_id.as_str(), topic, topic_type, self.caller_api.as_str()])
    }

    pub fn unregister_subscriber(&self, topic: &str) -> rosxmlrpc::client::Member {
        self.client.request("unregisterSubscriber",
                            &[self.client_id.as_str(), topic, self.caller_api.as_str()])
    }

    pub fn register_publisher(&self, topic: &str, topic_type: &str) -> rosxmlrpc::client::Member {
        self.client.request("registerPublisher",
                            &[self.client_id.as_str(), topic, topic_type, self.caller_api.as_str()])
    }

    pub fn unregister_publisher(&self, topic: &str) -> rosxmlrpc::client::Member {
        self.client.request("unregisterPublisher",
                            &[self.client_id.as_str(), topic, self.caller_api.as_str()])
    }
}
