use super::error::Result;
use super::master::Master;
use super::slave::Slave;
use super::clock::Clock;
use rosxmlrpc::Response;
use std::sync::Arc;
use std::sync::atomic::AtomicUsize;
use tcpros::{Message, PublisherStream, ServicePair, ServiceResult};

#[derive(Clone)]
pub struct Publisher<T: Message> {
    clock: Arc<Clock>,
    seq: Arc<AtomicUsize>,
    stream: PublisherStream<T>,
    _raii: Arc<InteractorRaii<PublisherInfo>>,
}

impl<T: Message> Publisher<T> {
    pub(crate) fn new(
        master: Arc<Master>,
        slave: Arc<Slave>,
        clock: Arc<Clock>,
        hostname: &str,
        name: &str,
    ) -> Result<Self> {
        let stream = slave.add_publication::<T>(hostname, name)?;
        let mut info = PublisherInfo {
            master,
            slave,
            name: name.into(),
        };

        match info.master.register_publisher(name, &T::msg_type()) {
            Ok(_) => Ok(Self {
                stream,
                clock,
                seq: Arc::new(AtomicUsize::new(0)),
                _raii: Arc::new(InteractorRaii::new(info)),
            }),
            Err(error) => {
                error!(
                    "Failed to register publisher for topic '{}': {}",
                    name, error
                );
                info.unregister()?;
                Err(error.into())
            }
        }
    }

    #[inline]
    pub fn send(&mut self, mut message: T) -> Result<()> {
        message.set_header(&self.clock, &self.seq);
        self.stream.send(&message).map_err(|v| v.into())
    }
}

struct PublisherInfo {
    master: Arc<Master>,
    slave: Arc<Slave>,
    name: String,
}

impl Interactor for PublisherInfo {
    fn unregister(&mut self) -> Response<()> {
        self.slave.remove_publication(&self.name);
        self.master.unregister_publisher(&self.name).map(|_| ())
    }
}

#[derive(Clone)]
pub struct Subscriber {
    _raii: Arc<InteractorRaii<SubscriberInfo>>,
}

impl Subscriber {
    pub(crate) fn new<T: Message, F: Fn(T) -> () + Send + 'static>(
        master: Arc<Master>,
        slave: Arc<Slave>,
        name: &str,
        callback: F,
    ) -> Result<Self> {
        slave.add_subscription::<T, F>(name, callback)?;

        match master.register_subscriber(name, &T::msg_type()) {
            Ok(publishers) => {
                if let Err(err) = slave.add_publishers_to_subscription(name, publishers.into_iter())
                {
                    error!(
                        "Failed to subscribe to all publishers of topic '{}': {}",
                        name, err
                    );
                }
                Ok(Self {
                    _raii: Arc::new(InteractorRaii::new(SubscriberInfo {
                        master,
                        slave,
                        name: name.into(),
                    })),
                })
            }
            Err(err) => {
                SubscriberInfo {
                    master,
                    slave,
                    name: name.into(),
                }.unregister()?;
                Err(err.into())
            }
        }
    }
}

struct SubscriberInfo {
    master: Arc<Master>,
    slave: Arc<Slave>,
    name: String,
}

impl Interactor for SubscriberInfo {
    fn unregister(&mut self) -> Response<()> {
        self.slave.remove_subscription(&self.name);
        self.master.unregister_subscriber(&self.name).map(|_| ())
    }
}

#[derive(Clone)]
pub struct Service {
    _raii: Arc<InteractorRaii<ServiceInfo>>,
}

impl Service {
    pub(crate) fn new<T, F>(
        master: Arc<Master>,
        slave: Arc<Slave>,
        hostname: &str,
        name: &str,
        handler: F,
    ) -> Result<Self>
    where
        T: ServicePair,
        F: Fn(T::Request) -> ServiceResult<T::Response> + Send + Sync + 'static,
    {
        let api = slave.add_service::<T, F>(hostname, name, handler)?;

        let mut info = ServiceInfo {
            master,
            slave,
            api,
            name: name.into(),
        };

        if let Err(err) = info.master.register_service(name, &info.api) {
            info.unregister()?;
            Err(err.into())
        } else {
            Ok(Self {
                _raii: Arc::new(InteractorRaii::new(info)),
            })
        }
    }
}

struct ServiceInfo {
    master: Arc<Master>,
    slave: Arc<Slave>,
    name: String,
    api: String,
}

impl Interactor for ServiceInfo {
    fn unregister(&mut self) -> Response<()> {
        self.slave.remove_service(&self.name);
        self.master
            .unregister_service(&self.name, &self.api)
            .map(|_| ())
    }
}

trait Interactor {
    fn unregister(&mut self) -> Response<()>;
}

struct InteractorRaii<I: Interactor> {
    interactor: I,
}

impl<I: Interactor> InteractorRaii<I> {
    pub fn new(interactor: I) -> InteractorRaii<I> {
        Self { interactor }
    }
}

impl<I: Interactor> Drop for InteractorRaii<I> {
    fn drop(&mut self) {
        if let Err(e) = self.interactor.unregister() {
            error!("Error while unloading: {:?}", e);
        }
    }
}
