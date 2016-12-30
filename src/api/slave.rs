use rosxmlrpc;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::sync::mpsc::channel;
use std::thread;
use super::error::ServerError as Error;
use super::slavehandler::{add_publishers_to_subscription, SlaveHandler};
use tcpros::{self, Message, Publisher, PublisherStream, Subscriber, Service, ServicePair};

pub struct Slave {
    name: String,
    uri: String,
    publications: Arc<Mutex<HashMap<String, Publisher>>>,
    subscriptions: Arc<Mutex<HashMap<String, Subscriber>>>,
    services: Arc<Mutex<HashMap<String, Service>>>,
}

type SerdeResult<T> = Result<T, Error>;

impl Slave {
    pub fn new(master_uri: &str, server_uri: &str, name: &str) -> Result<Slave, Error> {
        let (shutdown_tx, shutdown_rx) = channel();
        let handler = SlaveHandler::new(master_uri, name, shutdown_tx);
        let pubs = handler.publications.clone();
        let subs = handler.subscriptions.clone();
        let services = handler.services.clone();
        let mut server = rosxmlrpc::Server::new(server_uri, handler)?;
        let uri = server.uri.clone();
        thread::spawn(move || {
            match shutdown_rx.recv() {
                Ok(..) => info!("ROS Slave API shutdown by remote request"),
                Err(..) => info!("ROS Slave API shutdown by ROS client destruction"),
            };
            if let Err(err) = server.shutdown() {
                info!("Error during ROS Slave API shutdown: {}", err);
            }
        });
        Ok(Slave {
            name: String::from(name),
            uri: uri,
            publications: pubs,
            subscriptions: subs,
            services: services,
        })
    }

    pub fn uri(&self) -> &str {
        return &self.uri;
    }

    pub fn add_publishers_to_subscription<T>(&mut self,
                                             topic: &str,
                                             publishers: T)
                                             -> SerdeResult<()>
        where T: Iterator<Item = String>
    {
        add_publishers_to_subscription(&mut self.subscriptions.lock().unwrap(),
                                       &self.name,
                                       topic,
                                       publishers)
    }

    pub fn add_service<T, F>(&mut self,
                             hostname: &str,
                             service: &str,
                             handler: F)
                             -> SerdeResult<String>
        where T: ServicePair,
              F: Fn(T::Request) -> T::Response + Copy + Send + 'static
    {
        use std::collections::hash_map::Entry;
        match self.services.lock().unwrap().entry(String::from(service)) {
            Entry::Occupied(..) => {
                error!("Duplicate initiation of service '{}' attempted", service);
                Err(Error::Critical(String::from("Could not add duplicate service")))
            }
            Entry::Vacant(entry) => {
                let service = Service::new::<T, _, _>(format!("{}:0", hostname).as_str(),
                                                      service,
                                                      &self.name,
                                                      handler)?;
                let api = format!("{}:{}", service.ip, service.port);
                entry.insert(service);
                Ok(api)
            }
        }
    }

    pub fn remove_service(&mut self, service: &str) {
        self.services.lock().unwrap().remove(service);
    }

    pub fn add_publication<T>(&mut self,
                              hostname: &str,
                              topic: &str)
                              -> Result<PublisherStream<T>, tcpros::Error>
        where T: Message
    {
        use std::collections::hash_map::Entry;
        match self.publications.lock().unwrap().entry(String::from(topic)) {
            Entry::Occupied(publisher_entry) => publisher_entry.get().stream(),
            Entry::Vacant(entry) => {
                let publisher = Publisher::new::<T, _>(format!("{}:0", hostname).as_str(), topic)?;
                entry.insert(publisher).stream()
            }
        }
    }

    pub fn remove_publication(&mut self, topic: &str) {
        self.publications.lock().unwrap().remove(topic);
    }

    pub fn add_subscription<T, F>(&mut self, topic: &str, callback: F) -> Result<(), Error>
        where T: Message,
              F: Fn(T) -> () + Send + 'static
    {
        use std::collections::hash_map::Entry;
        match self.subscriptions.lock().unwrap().entry(String::from(topic)) {
            Entry::Occupied(..) => {
                error!("Duplicate subscription to topic '{}' attempted", topic);
                Err(Error::Critical(String::from("Could not add duplicate subscription to topic")))
            }
            Entry::Vacant(entry) => {
                let subscriber = Subscriber::new::<T, F>(&self.name, topic, callback);
                entry.insert(subscriber);
                Ok(())
            }
        }
    }

    pub fn remove_subscription(&mut self, topic: &str) {
        self.subscriptions.lock().unwrap().remove(topic);
    }
}
