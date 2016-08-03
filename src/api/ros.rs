use std;
use super::{Master, Slave};

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
}
