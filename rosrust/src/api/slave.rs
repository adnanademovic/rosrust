use futures::sync::mpsc::channel as futures_channel;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::sync::mpsc::{channel, Sender};
use std::thread;
use super::error::{self, ErrorKind, Result, ResultExt};
use super::slavehandler::{add_publishers_to_subscription, SlaveHandler};
use tcpros::{Message, Publisher, PublisherStream, Service, ServicePair, ServiceResult, Subscriber};

pub struct Slave {
    name: String,
    uri: String,
    pub publications: Arc<Mutex<HashMap<String, Publisher>>>,
    pub subscriptions: Arc<Mutex<HashMap<String, Subscriber>>>,
    pub services: Arc<Mutex<HashMap<String, Service>>>,
}

type SerdeResult<T> = Result<T>;

impl Slave {
    pub fn new(
        master_uri: &str,
        hostname: &str,
        port: u16,
        name: &str,
        outer_shutdown_tx: Sender<()>,
    ) -> Result<Slave> {
        use std::net::ToSocketAddrs;
        use futures::{Future, Stream};

        let (shutdown_tx, shutdown_rx) = futures_channel(1);
        let handler = SlaveHandler::new(master_uri, hostname, name, shutdown_tx);
        let pubs = Arc::clone(&handler.publications);
        let subs = Arc::clone(&handler.subscriptions);
        let services = Arc::clone(&handler.services);
        let (port_tx, port_rx) = channel();
        let socket_addr = match (hostname, port).to_socket_addrs()?.next() {
            Some(socket_addr) => socket_addr,
            None => bail!("Bad address provided: {}:{}", hostname, port),
        };

        thread::spawn(move || {
            let bound_handler = match handler.bind(&socket_addr) {
                Ok(v) => v,
                Err(err) => {
                    port_tx.send(Err(err)).expect(FAILED_TO_LOCK);
                    return;
                }
            };
            let port = bound_handler.local_addr().map(|v| v.port());
            port_tx.send(port).expect(FAILED_TO_LOCK);
            let shutdown_future = shutdown_rx.into_future().map(|_| ()).map_err(|_| ());
            if let Err(err) = bound_handler.run_until(shutdown_future) {
                info!("Error during ROS Slave API initiation: {}", err);
            }
            outer_shutdown_tx.send(()).is_ok();
        });

        let port = port_rx
            .recv()
            .expect(FAILED_TO_LOCK)
            .chain_err(|| "Failed to get port")?;
        let uri = format!("http://{}:{}/", hostname, port);

        Ok(Slave {
            name: String::from(name),
            uri: uri,
            publications: pubs,
            subscriptions: subs,
            services: services,
        })
    }

    #[inline]
    pub fn uri(&self) -> &str {
        &self.uri
    }

    pub fn add_publishers_to_subscription<T>(&self, topic: &str, publishers: T) -> SerdeResult<()>
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
        &self,
        hostname: &str,
        service: &str,
        handler: F,
    ) -> SerdeResult<String>
    where
        T: ServicePair,
        F: Fn(T::Request) -> ServiceResult<T::Response> + Send + Sync + 'static,
    {
        use std::collections::hash_map::Entry;
        match self.services
            .lock()
            .expect(FAILED_TO_LOCK)
            .entry(String::from(service))
        {
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

    pub fn remove_service(&self, service: &str) {
        self.services.lock().expect(FAILED_TO_LOCK).remove(service);
    }

    pub fn add_publication<T>(
        &self,
        hostname: &str,
        topic: &str,
    ) -> error::tcpros::Result<PublisherStream<T>>
    where
        T: Message,
    {
        use std::collections::hash_map::Entry;
        match self.publications
            .lock()
            .expect(FAILED_TO_LOCK)
            .entry(String::from(topic))
        {
            Entry::Occupied(publisher_entry) => publisher_entry.get().stream(),
            Entry::Vacant(entry) => {
                let publisher = Publisher::new::<T, _>(format!("{}:0", hostname).as_str(), topic)?;
                entry.insert(publisher).stream()
            }
        }
    }

    pub fn remove_publication(&self, topic: &str) {
        self.publications
            .lock()
            .expect(FAILED_TO_LOCK)
            .remove(topic);
    }

    pub fn add_subscription<T, F>(&self, topic: &str, callback: F) -> Result<()>
    where
        T: Message,
        F: Fn(T) -> () + Send + 'static,
    {
        use std::collections::hash_map::Entry;
        match self.subscriptions
            .lock()
            .expect(FAILED_TO_LOCK)
            .entry(String::from(topic))
        {
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

    pub fn remove_subscription(&self, topic: &str) {
        self.subscriptions
            .lock()
            .expect(FAILED_TO_LOCK)
            .remove(topic);
    }
}

static FAILED_TO_LOCK: &'static str = "Failed to acquire lock";
