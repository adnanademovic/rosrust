use rustc_serialize::{Encodable, Decodable};
use std;
use nix::unistd::gethostname;
use super::master::Master;
use super::slave::Slave;
use super::error::ServerError;
use rosxmlrpc::error::Error;
use tcpros::{Message, Publisher};

pub struct Ros {
    pub master: Master,
    pub slave: Slave,
    hostname: String,
}

impl Ros {
    pub fn new(name: &str) -> Result<Ros, ServerError> {
        let master_uri = std::env::var("ROS_MASTER_URI")
            .unwrap_or("http://localhost:11311/".to_owned());
        let mut hostname = vec![];
        gethostname(&mut hostname)?;
        let hostname = String::from_utf8(hostname)?;
        let slave = Slave::new(&master_uri, &format!("{}:0", hostname), name)?;
        let master = Master::new(&master_uri, name, &slave.uri());
        Ok(Ros {
            master: master,
            slave: slave,
            hostname: hostname,
        })
    }

    pub fn node_uri(&self) -> &str {
        return self.slave.uri();
    }

    pub fn get_param<T: Decodable>(&self, name: &str) -> Result<T, Error> {
        self.master.get_param::<T>(name)
    }

    pub fn set_param<T: Encodable>(&self, name: &str, value: &T) -> Result<(), Error> {
        self.master.set_param::<T>(name, value).and(Ok(()))
    }

    pub fn has_param(&self, name: &str) -> Result<bool, Error> {
        self.master.has_param(name)
    }

    pub fn get_param_names(&self) -> Result<Vec<String>, Error> {
        self.master.get_param_names()
    }

    pub fn spin(&mut self) -> Result<(), String> {
        self.slave.handle_calls()
    }

    pub fn subscribe<T, F>(&mut self, topic: &str, callback: F) -> Result<(), ServerError>
        where T: Message + Decodable,
              F: Fn(T) -> () + Send + 'static
    {
        self.slave
            .add_subscription::<T, F>(topic, callback)
            .ok_or(ServerError::Critical(String::from("Could not add duplicate subscription to \
                                                       topic")))?;

        match self.master.register_subscriber(topic, &T::msg_type()) {
            Ok(publishers) => {
                let topic = topic.to_owned();
                let subscription = self.slave.get_subscription(&topic).unwrap();
                for publisher in publishers {
                    if let Err(err) = subscription.connect_to(publisher.as_str()) {
                        error!("ROS provided illegal publisher name '{}': {}",
                               publisher,
                               err);
                    }
                }
                Ok(())
            }
            Err(err) => {
                self.slave.remove_subscription(topic);
                Err(ServerError::XmlRpc(err))
            }
        }
    }

    pub fn publish<T>(&mut self, topic: &str) -> Result<&Publisher, ServerError>
        where T: Message + Encodable
    {
        self.slave.add_publication::<T>(&self.hostname, topic)?;
        match self.master.register_publisher(topic, &T::msg_type()) {
            Ok(_) => Ok(self.slave.get_publication(topic).unwrap()),
            Err(error) => {
                error!("Failed to register publisher for topic '{}': {}",
                       topic,
                       error);
                self.slave.remove_publication(topic);
                self.master.unregister_publisher(topic)?;
                Err(ServerError::XmlRpc(error))
            }
        }
    }
}
