mod handler;
mod publications;
mod subscriptions;

use self::handler::SlaveHandler;
use super::error::{self, ErrorKind, Result, ResultExt};
use crossbeam::channel::{unbounded, Sender};
use futures::sync::mpsc::channel as futures_channel;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::thread;
use tcpros::{Message, PublisherStream, Service, ServicePair, ServiceResult};

pub struct Slave {
    name: String,
    uri: String,
    pub publications: publications::PublicationsTracker,
    pub subscriptions: subscriptions::SubscriptionsTracker,
    pub services: Arc<Mutex<HashMap<String, Service>>>,
}

type SerdeResult<T> = Result<T>;

impl Slave {
    pub fn new(
        master_uri: &str,
        hostname: &str,
        bind_address: &str,
        port: u16,
        name: &str,
        outer_shutdown_tx: Sender<()>,
    ) -> Result<Slave> {
        use futures::{Future, Stream};
        use std::net::ToSocketAddrs;

        let (shutdown_tx, shutdown_rx) = futures_channel(1);
        let handler = SlaveHandler::new(master_uri, hostname, name, shutdown_tx);
        let publications = handler.publications.clone();
        let subscriptions = handler.subscriptions.clone();
        let services = Arc::clone(&handler.services);
        let (port_tx, port_rx) = unbounded();
        let socket_addr = match (bind_address, port).to_socket_addrs()?.next() {
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
            uri,
            publications,
            subscriptions,
            services,
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
        self.subscriptions
            .add_publishers(topic, &self.name, publishers)
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
        match self
            .services
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

    #[inline]
    pub fn remove_service(&self, service: &str) {
        self.services.lock().expect(FAILED_TO_LOCK).remove(service);
    }

    #[inline]
    pub fn add_publication<T>(
        &self,
        hostname: &str,
        topic: &str,
    ) -> error::tcpros::Result<PublisherStream<T>>
    where
        T: Message,
    {
        self.publications.add(hostname, topic)
    }

    #[inline]
    pub fn remove_publication(&self, topic: &str) {
        self.publications.remove(topic)
    }

    #[inline]
    pub fn add_subscription<T, F>(&self, topic: &str, callback: F) -> Result<()>
    where
        T: Message,
        F: Fn(T) -> () + Send + 'static,
    {
        self.subscriptions.add(&self.name, topic, callback)
    }

    #[inline]
    pub fn remove_subscription(&self, topic: &str) {
        self.subscriptions.remove(topic)
    }
}

static FAILED_TO_LOCK: &'static str = "Failed to acquire lock";
