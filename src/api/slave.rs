use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::sync::mpsc::channel;
use std::thread;
use super::error::{self, ErrorKind, Result};
use super::slavehandler::{add_publishers_to_subscription, SlaveHandler};
use tcpros::{Message, Publisher, PublisherStream, Subscriber, Service, ServicePair, ServiceResult};

pub struct Slave {
    name: String,
    uri: String,
    publications: Arc<Mutex<HashMap<String, Publisher>>>,
    subscriptions: Arc<Mutex<HashMap<String, Subscriber>>>,
    services: Arc<Mutex<HashMap<String, Service>>>,
}

type SerdeResult<T> = Result<T>;

impl Slave {
    pub fn new(master_uri: &str, hostname: &str, port: u16, name: &str) -> Result<Slave> {
        use std::net::ToSocketAddrs;

        let (shutdown_tx, _shutdown_rx) = channel();
        let handler = SlaveHandler::new(master_uri, hostname, name, shutdown_tx);
        let pubs = handler.publications.clone();
        let subs = handler.subscriptions.clone();
        let services = handler.services.clone();
        // TODO: allow OS assigned port numbers
        let uri = format!("http://{}:{}/", hostname, port);
        let socket_addr = match (hostname, port).to_socket_addrs()?.next() {
            Some(socket_addr) => socket_addr,
            None => bail!("Bad address provided: {}:{}", hostname, port),
        };
        thread::spawn(move || if let Err(err) = handler.run(&socket_addr) {
            info!("Error during ROS Slave API initiation: {}", err);
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

    pub fn add_publishers_to_subscription<T>(
        &mut self,
        topic: &str,
        publishers: T,
    ) -> SerdeResult<()>
    where
        T: Iterator<Item = String>,
    {
        add_publishers_to_subscription(
            &mut self.subscriptions.lock().expect(FAILED_TO_LOCK),
            &self.name,
            topic,
            publishers,
        )
    }

    pub fn add_service<T, F>(
        &mut self,
        hostname: &str,
        service: &str,
        handler: F,
    ) -> SerdeResult<String>
    where
        T: ServicePair,
        F: Fn(T::Request) -> ServiceResult<T::Response> + Send + Sync + 'static,
    {
        use std::collections::hash_map::Entry;
        match self.services.lock().expect(FAILED_TO_LOCK).entry(
            String::from(
                service,
            ),
        ) {
            Entry::Occupied(..) => {
                error!("Duplicate initiation of service '{}' attempted", service);
                Err(ErrorKind::Duplicate("service".into()).into())
            }
            Entry::Vacant(entry) => {
                let service = Service::new::<T, _>(hostname, 0, service, &self.name, handler)?;
                let api = service.api.clone();
                entry.insert(service);
                Ok(api)
            }
        }
    }

    pub fn remove_service(&mut self, service: &str) {
        self.services.lock().expect(FAILED_TO_LOCK).remove(service);
    }

    pub fn add_publication<T>(
        &mut self,
        hostname: &str,
        topic: &str,
    ) -> error::tcpros::Result<PublisherStream<T>>
    where
        T: Message,
    {
        use std::collections::hash_map::Entry;
        match self.publications.lock().expect(FAILED_TO_LOCK).entry(
            String::from(topic),
        ) {
            Entry::Occupied(publisher_entry) => publisher_entry.get().stream(),
            Entry::Vacant(entry) => {
                let publisher = Publisher::new::<T, _>(format!("{}:0", hostname).as_str(), topic)?;
                entry.insert(publisher).stream()
            }
        }
    }

    pub fn remove_publication(&mut self, topic: &str) {
        self.publications.lock().expect(FAILED_TO_LOCK).remove(
            topic,
        );
    }

    pub fn add_subscription<T, F>(&mut self, topic: &str, callback: F) -> Result<()>
    where
        T: Message,
        F: Fn(T) -> () + Send + 'static,
    {
        use std::collections::hash_map::Entry;
        match self.subscriptions.lock().expect(FAILED_TO_LOCK).entry(
            String::from(topic),
        ) {
            Entry::Occupied(..) => {
                error!("Duplicate subscription to topic '{}' attempted", topic);
                Err(ErrorKind::Duplicate("subscription".into()).into())
            }
            Entry::Vacant(entry) => {
                let subscriber = Subscriber::new::<T, F>(&self.name, topic, callback);
                entry.insert(subscriber);
                Ok(())
            }
        }
    }

    pub fn remove_subscription(&mut self, topic: &str) {
        self.subscriptions.lock().expect(FAILED_TO_LOCK).remove(
            topic,
        );
    }
}

static FAILED_TO_LOCK: &'static str = "Failed to acquire lock";
