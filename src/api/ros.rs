use rustc_serialize::{Encodable, Decodable};
use std;
use nix::unistd::gethostname;
use super::master::{self, Master, MasterResult};
use super::slave::Slave;
use super::error::ServerError;
use super::value::Topic;
use super::naming::Resolver;
use tcpros::{Message, PublisherStream};
use rosxmlrpc::serde::XmlRpcValue;

pub struct Ros {
    master: Master,
    slave: Slave,
    hostname: String,
    resolver: Resolver,
}

impl Ros {
    pub fn new(name: &str) -> Result<Ros, ServerError> {
        let namespace = std::env::var("ROS_NAMESPACE").unwrap_or(String::from(""));
        Ros::new_raw(&namespace, name)
    }

    pub fn new_raw(namespace: &str, name: &str) -> Result<Ros, ServerError> {
        let master_uri = std::env::var("ROS_MASTER_URI")
            .unwrap_or(String::from("http://localhost:11311/"));
        let mut hostname = vec![];
        gethostname(&mut hostname)?;
        let hostname = String::from_utf8(hostname)?;
        let slave = Slave::new(&master_uri, &format!("{}:0", hostname), name)?;
        let master = Master::new(&master_uri, name, &slave.uri());
        let resolver = Resolver::new(&format!("{}/{}", namespace, name))?;
        Ok(Ros {
            master: master,
            slave: slave,
            hostname: hostname,
            resolver: resolver,
        })
    }

    pub fn map(&mut self, source: &str, destination: &str) -> Result<(), ServerError> {
        self.resolver.map(source, destination).map_err(|v| ServerError::Naming(v))
    }

    pub fn node_uri(&self) -> &str {
        return self.slave.uri();
    }

    pub fn param<'a, 'b>(&'a self, name: &'b str) -> Option<Parameter<'a>> {
        self.resolver.translate(name).ok().map(|v| {
            Parameter {
                master: &self.master,
                name: v,
            }
        })
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

    pub fn service<Treq, Tres, F>(&mut self, service: &str, handler: F) -> Result<(), ServerError>
        where Treq: Message + Decodable,
              Tres: Message + Encodable,
              F: Fn(Treq) -> Tres + Copy + Send + 'static
    {
        let name = self.resolver.translate(service)?;
        let api = self.slave.add_service::<Treq, Tres, F>(&self.hostname, &name, handler)?;

        if let Err(err) = self.master.register_service(&name, &api) {
            self.slave.remove_service(&name);
            self.master.unregister_service(&name, &api)?;
            Err(ServerError::from(err))
        } else {
            Ok(())
        }
    }

    pub fn subscribe<T, F>(&mut self, topic: &str, callback: F) -> Result<(), ServerError>
        where T: Message + Decodable,
              F: Fn(T) -> () + Send + 'static
    {
        let name = self.resolver.translate(topic)?;
        self.slave.add_subscription::<T, F>(&name, callback)?;

        match self.master.register_subscriber(&name, &T::msg_type()) {
            Ok(publishers) => {
                if let Err(err) = self.slave
                    .add_publishers_to_subscription(&name, publishers.into_iter()) {
                    error!("Failed to subscribe to all publishers of topic '{}': {}",
                           name,
                           err);
                }
                Ok(())
            }
            Err(err) => {
                self.slave.remove_subscription(&name);
                self.master.unregister_subscriber(&name)?;
                Err(ServerError::from(err))
            }
        }
    }

    pub fn publish<T>(&mut self, topic: &str) -> Result<PublisherStream<T>, ServerError>
        where T: Message + Encodable
    {
        let name = self.resolver.translate(topic)?;
        let stream = self.slave.add_publication::<T>(&self.hostname, &name)?;
        match self.master.register_publisher(&name, &T::msg_type()) {
            Ok(_) => Ok(stream),
            Err(error) => {
                error!("Failed to register publisher for topic '{}': {}",
                       name,
                       error);
                self.slave.remove_publication(&name);
                self.master.unregister_publisher(&name)?;
                Err(ServerError::from(error))
            }
        }
    }
}

pub struct Parameter<'a> {
    master: &'a Master,
    name: String,
}

impl<'a> Parameter<'a> {
    pub fn get<T: Decodable>(&self) -> MasterResult<T> {
        self.master.get_param::<T>(&self.name)
    }

    pub fn get_raw(&self) -> MasterResult<XmlRpcValue> {
        self.master.get_param_any(&self.name)
    }

    pub fn set<T: Encodable>(&self, value: &T) -> MasterResult<()> {
        self.master.set_param::<T>(&self.name, value).and(Ok(()))
    }

    pub fn delete(&self) -> MasterResult<()> {
        self.master.delete_param(&self.name).and(Ok(()))
    }

    pub fn exists(&self) -> MasterResult<bool> {
        self.master.has_param(&self.name)
    }

    pub fn search(&self) -> MasterResult<String> {
        self.master.search_param(&self.name)
    }
}
