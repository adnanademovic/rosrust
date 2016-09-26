use rustc_serialize::{Encodable, Decodable};
use std::sync::mpsc;
use std::thread;
use std;
use super::master::Master;
use super::slave::Slave;
use super::error::ServerError;
use rosxmlrpc::error::Error;
use rosxmlrpc;
use tcpros::message::RosMessage;
use tcpros;

pub struct Ros {
    pub master: Master,
    pub slave: Slave,
    name: String,
}

impl Ros {
    pub fn new(hostname: &str, name: &str) -> Result<Ros, ServerError> {
        let master_uri = std::env::var("ROS_MASTER_URI")
            .unwrap_or("http://localhost:11311/".to_owned());
        let slave = try!(Slave::new(&master_uri, &format!("{}:0", hostname)));
        let master = Master::new(&master_uri, name, &slave.uri());
        Ok(Ros {
            master: master,
            slave: slave,
            name: name.to_owned(),
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

    pub fn subscribe<T>(&mut self, topic: &str) -> Result<mpsc::Receiver<T>, Error>
        where T: RosMessage + Decodable + Send + 'static
    {
        if let Some(rx_publishers) = self.slave.add_subscription(topic, &T::msg_type()) {
            match self.master.register_subscriber(topic, &T::msg_type()) {
                Ok(publishers) => {
                    let (tx, rx) = mpsc::channel::<T>();
                    let name = self.name.clone();
                    let topic = topic.to_owned();
                    thread::spawn(move || {
                        // Spawn new subscription connection thread for each publisher
                        for publisher in publishers.into_iter().chain(rx_publishers.into_iter()) {
                            let tx = tx.clone();
                            let name = name.clone();
                            let topic = topic.clone();
                            thread::spawn(move || {
                                connect_subscriber(&name, &publisher, &topic, tx)
                            });
                        }
                    });
                    Ok(rx)
                }
                Err(err) => {
                    self.slave.remove_subscription(topic);
                    Err(err)
                }
            }
        } else {
            panic!("Handle this path");
        }
    }
}

fn connect_subscriber<T>(caller_id: &str, publisher: &str, topic: &str, tx: mpsc::Sender<T>)
    where T: RosMessage + Decodable + Send + 'static
{
    let (protocol, hostname, port) = request_topic(publisher, caller_id, topic).unwrap();
    if protocol != "TCPROS" {
        // This should never happen, due to the nature of ROS
        panic!("Protocol does not match");
    }
    let publisher = format!("{}:{}", hostname, port);
    let subscriber = tcpros::subscriber::Subscriber::<T>::new(publisher.as_str(), caller_id, topic)
        .unwrap();
    for val in subscriber {
        if let Err(_) = tx.send(val) {
            break;
        }
    }
}

fn request_topic(publisher_uri: &str,
                 caller_id: &str,
                 topic: &str)
                 -> Result<(String, String, i32), rosxmlrpc::error::Error> {
    let protocols = try!(rosxmlrpc::Client::new(publisher_uri)
            .request_long::<(i32, String, (String, String, i32)), Vec<Vec<&str>>>(
                "requestTopic", &[caller_id, topic], Some(&vec![vec!["TCPROS"]])));
    Ok(protocols.2)
}
