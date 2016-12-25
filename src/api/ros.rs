use rustc_serialize::{Encodable, Decodable};
use std;
use nix::unistd::gethostname;
use super::master::{self, Master, MasterResult};
use super::slave::Slave;
use super::error::ServerError;
use super::value::Topic;
use tcpros::{Message, PublisherStream};
use rosxmlrpc::serde::XmlRpcValue;

pub struct Ros {
    master: Master,
    slave: Slave,
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

    pub fn param<'a, 'b>(&'a self, name: &'b str) -> Parameter<'a, 'b> {
        Parameter {
            master: &self.master,
            name: name,
        }
    }

    pub fn parameters(&self) -> MasterResult<Vec<String>> {
        self.master.get_param_names()
    }

    pub fn state(&self) -> MasterResult<master::SystemState> {
        self.master.get_system_state()
    }

    pub fn topics(&self) -> MasterResult<Vec<Topic>> {
        self.master.get_topic_types()
    }

    pub fn subscribe<T, F>(&mut self, topic: &str, callback: F) -> Result<(), ServerError>
        where T: Message + Decodable,
              F: Fn(T) -> () + Send + 'static
    {
        self.slave.add_subscription::<T, F>(topic, callback)?;

        match self.master.register_subscriber(topic, &T::msg_type()) {
            Ok(publishers) => {
                if let Err(err) = self.slave
                    .add_publishers_to_subscription(topic, publishers.into_iter()) {
                    error!("Failed to subscribe to all publishers of topic '{}': {}",
                           topic,
                           err);
                }
                Ok(())
            }
            Err(err) => {
                self.slave.remove_subscription(topic);
                self.master.unregister_subscriber(topic)?;
                Err(ServerError::from(err))
            }
        }
    }

    pub fn publish<T>(&mut self, topic: &str) -> Result<PublisherStream<T>, ServerError>
        where T: Message + Encodable
    {
        let stream = self.slave.add_publication::<T>(&self.hostname, topic)?;
        match self.master.register_publisher(topic, &T::msg_type()) {
            Ok(_) => Ok(stream),
            Err(error) => {
                error!("Failed to register publisher for topic '{}': {}",
                       topic,
                       error);
                self.slave.remove_publication(topic);
                self.master.unregister_publisher(topic)?;
                Err(ServerError::from(error))
            }
        }
    }
}

pub struct Parameter<'a, 'b> {
    master: &'a Master,
    name: &'b str,
}

impl<'a, 'b> Parameter<'a, 'b> {
    pub fn get<T: Decodable>(&self) -> MasterResult<T> {
        self.master.get_param::<T>(self.name)
    }

    pub fn get_raw(&self) -> MasterResult<XmlRpcValue> {
        self.master.get_param_any(self.name)
    }

    pub fn set<T: Encodable>(&self, value: &T) -> MasterResult<()> {
        self.master.set_param::<T>(self.name, value).and(Ok(()))
    }

    pub fn delete(&self) -> MasterResult<()> {
        self.master.delete_param(self.name).and(Ok(()))
    }

    pub fn exists(&self) -> MasterResult<bool> {
        self.master.has_param(self.name)
    }

    pub fn search(&self) -> MasterResult<String> {
        self.master.search_param(self.name)
    }
}
