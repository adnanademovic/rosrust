mod handler;
mod publications;
mod subscriptions;

use self::handler::SlaveHandler;
use super::error::{self, ErrorKind, Result};
use crate::tcpros::{Message, PublisherStream, Service, ServicePair, ServiceResult};
use crate::util::{FAILED_TO_LOCK, MPSC_CHANNEL_UNEXPECTEDLY_CLOSED};
use crossbeam::channel::{bounded, unbounded, Sender, TryRecvError};
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
    pub shutdown_tx: Sender<()>,
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
        use std::net::ToSocketAddrs;

        let (shutdown_tx, shutdown_rx) = bounded(0);
        let handler = SlaveHandler::new(master_uri, hostname, name, shutdown_tx.clone());
        let publications = handler.publications.clone();
        let subscriptions = handler.subscriptions.clone();
        let services = Arc::clone(&handler.services);
        let (port_tx, port_rx) = unbounded();
        let socket_addr = match (bind_address, port).to_socket_addrs()?.next() {
            Some(socket_addr) => socket_addr,
            None => bail!(error::ErrorKind::from(error::rosxmlrpc::ErrorKind::BadUri(
                format!("{}:{}", hostname, port)
            ))),
        };

        thread::spawn(move || {
            let bound_handler = match handler.bind(&socket_addr) {
                Ok(v) => v,
                Err(err) => {
                    port_tx
                        .send(Err(err))
                        .expect(MPSC_CHANNEL_UNEXPECTEDLY_CLOSED);
                    return;
                }
            };
            let port = bound_handler.local_addr().port();
            port_tx
                .send(Ok(port))
                .expect(MPSC_CHANNEL_UNEXPECTEDLY_CLOSED);
            loop {
                match shutdown_rx.try_recv() {
                    Ok(_) | Err(TryRecvError::Disconnected) => break,
                    Err(TryRecvError::Empty) => {}
                }
                bound_handler.poll();
            }
            outer_shutdown_tx.send(()).is_ok();
        });

        let port = port_rx.recv().expect(MPSC_CHANNEL_UNEXPECTEDLY_CLOSED)?;
        let uri = format!("http://{}:{}/", hostname, port);

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
    ) -> error::tcpros::Result<PublisherStream<T>>
    where
        T: Message,
    {
        self.publications.add(hostname, topic, queue_size)
    }

    #[inline]
    pub fn remove_publication(&self, topic: &str) {
        self.publications.remove(topic)
    }

    #[inline]
    pub fn add_subscription<T, F>(&self, topic: &str, queue_size: usize, callback: F) -> Result<()>
    where
        T: Message,
        F: Fn(T) -> () + Send + 'static,
    {
        self.subscriptions
            .add(&self.name, topic, queue_size, callback)
    }

    #[inline]
    pub fn remove_subscription(&self, topic: &str) {
        self.subscriptions.remove(topic)
    }

    #[inline]
    pub fn get_publisher_count_of_subscription(&self, topic: &str) -> usize {
        self.subscriptions.publisher_count(topic)
    }
}
