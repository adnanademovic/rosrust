mod handler;
mod publications;
mod subscriptions;

pub use self::handler::ParamCache;
use self::handler::SlaveHandler;
use super::error::{self, ErrorKind, Result};
use crate::api::ShutdownManager;
use crate::tcpros::{Message, PublisherStream, Service, ServicePair, ServiceResult};
use crate::util::{kill, FAILED_TO_LOCK};
use crate::{RawMessageDescription, SubscriptionHandler};
use crossbeam::channel::TryRecvError;
use error_chain::bail;
use log::error;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::thread;

pub struct Slave {
    name: String,
    uri: String,
    pub publications: publications::PublicationsTracker,
    pub subscriptions: subscriptions::SubscriptionsTracker,
    pub services: Arc<Mutex<HashMap<String, Service>>>,
    pub shutdown_tx: kill::Sender,
}

type SerdeResult<T> = Result<T>;

impl Slave {
    pub fn new(
        master_uri: &str,
        hostname: &str,
        bind_address: &str,
        port: u16,
        name: &str,
        param_cache: ParamCache,
        shutdown_manager: Arc<ShutdownManager>,
    ) -> Result<Slave> {
        use std::net::ToSocketAddrs;

        let (shutdown_tx, shutdown_rx) = kill::channel(kill::KillMode::Sync);
        let handler =
            SlaveHandler::new(master_uri, hostname, name, param_cache, shutdown_tx.clone());
        let publications = handler.publications.clone();
        let subscriptions = handler.subscriptions.clone();
        let services = Arc::clone(&handler.services);
        let socket_addr = match (bind_address, port).to_socket_addrs()?.next() {
            Some(socket_addr) => socket_addr,
            None => bail!(error::ErrorKind::from(error::rosxmlrpc::ErrorKind::BadUri(
                format!("{}:{}", hostname, port)
            ))),
        };

        let bound_handler = handler.bind(&socket_addr)?;

        let port = bound_handler.local_addr().port();
        let uri = format!("http://{}:{}/", hostname, port);

        thread::spawn(move || {
            loop {
                match shutdown_rx.try_recv() {
                    Ok(_) | Err(TryRecvError::Disconnected) => break,
                    Err(TryRecvError::Empty) => {}
                }
                bound_handler.poll();
                // TODO: use a timed out poll once rouille provides it
                std::thread::sleep(std::time::Duration::from_millis(5));
            }
            shutdown_manager.shutdown();
        });

        Ok(Slave {
            name: String::from(name),
            uri,
            publications,
            subscriptions,
            services,
            shutdown_tx,
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
        bind_address: &str,
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
                let service =
                    Service::new::<T, _>(hostname, bind_address, 0, service, &self.name, handler)?;
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
        queue_size: usize,
        message_description: RawMessageDescription,
    ) -> error::tcpros::Result<PublisherStream<T>>
    where
        T: Message,
    {
        self.publications
            .add(hostname, topic, queue_size, &self.name, message_description)
    }

    #[inline]
    pub fn remove_publication(&self, topic: &str) {
        self.publications.remove(topic)
    }

    #[inline]
    pub fn add_subscription<T, H>(
        &self,
        topic: &str,
        queue_size: usize,
        handler: H,
    ) -> Result<usize>
    where
        T: Message,
        H: SubscriptionHandler<T>,
    {
        self.subscriptions
            .add(&self.name, topic, queue_size, handler)
    }

    #[inline]
    pub fn remove_subscription(&self, topic: &str, id: usize) {
        self.subscriptions.remove(topic, id)
    }

    #[inline]
    pub fn get_publisher_count_of_subscription(&self, topic: &str) -> usize {
        self.subscriptions.publisher_count(topic)
    }

    #[inline]
    pub fn get_publisher_uris_of_subscription(&self, topic: &str) -> Vec<String> {
        self.subscriptions.publisher_uris(topic)
    }
}
