use rustc_serialize::{Encodable, Decodable};
use std;
use super::master::Master;
use super::slave::Slave;
use super::slave;
use rosxmlrpc::error::Error;

pub struct Ros {
    pub master: Master,
    pub slave: Slave,
}

impl Ros {
    pub fn new(hostname: &str, name: &str) -> Result<Ros, super::slave::Error> {
        let master_uri = std::env::var("ROS_MASTER_URI")
            .unwrap_or("http://localhost:11311/".to_owned());
        let slave = try!(Slave::new(&master_uri, &format!("{}:0", hostname)));
        let master = Master::new(&master_uri, name, &slave.uri());
        Ok(Ros {
            master: master,
            slave: slave,
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

    pub fn spin(&mut self) -> Result<(), slave::Error> {
        self.slave.handle_call_queue().and(Ok(()))
    }
}
