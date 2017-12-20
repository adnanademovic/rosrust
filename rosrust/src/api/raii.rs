use super::error::Result;
use super::master::Master;
use super::slave::Slave;
use rosxmlrpc::Response;
use std::sync::Arc;
use tcpros::{Message, PublisherStream, ServicePair, ServiceResult};

#[derive(Clone)]
pub struct Publisher<'a, T: Message> {
    stream: PublisherStream<T>,
    _raii: Arc<InteractorRaii<PublisherInfo<'a>>>,
}

impl<'a, T: Message> Publisher<'a, T> {
    pub(crate) fn new(
        master: &'a Master,
        slave: &'a mut Slave,
        hostname: &str,
        name: &str,
    ) -> Result<Self> {
        let stream = slave.add_publication::<T>(hostname, name)?;
        let mut info = PublisherInfo {
            master,
            slave,
            name: name.into(),
        };

        match master.register_publisher(&name, &T::msg_type()) {
            Ok(_) => Ok(Self {
                stream,
                _raii: Arc::new(InteractorRaii::new(info)),
            }),
            Err(error) => {
                error!(
                    "Failed to register publisher for topic '{}': {}",
                    name,
                    error
                );
                info.unregister()?;
                Err(error.into())
            }
        }
    }

    #[inline]
    pub fn send(&mut self, message: T) -> Result<()> {
        self.stream.send(message).map_err(|v| v.into())
    }
}

struct PublisherInfo<'a> {
    master: &'a Master,
    slave: &'a Slave,
    name: String,
}

impl<'a> Interactor for PublisherInfo<'a> {
    fn unregister(&mut self) -> Response<()> {
        self.slave.remove_publication(&self.name);
        self.master.unregister_publisher(&self.name).map(|_| ())
    }
}

#[derive(Clone)]
pub struct Subscriber<'a> {
    _raii: Arc<InteractorRaii<SubscriberInfo<'a>>>,
}

impl<'a> Subscriber<'a> {
    pub(crate) fn new<T: Message, F: Fn(T) -> () + Send + 'static>(
        master: &'a Master,
        slave: &'a mut Slave,
        name: &str,
        callback: F,
    ) -> Result<Self> {
        slave.add_subscription::<T, F>(name, callback)?;

        match master.register_subscriber(name, &T::msg_type()) {
            Ok(publishers) => {
                if let Err(err) = slave.add_publishers_to_subscription(
                    &name,
                    publishers.into_iter(),
                )
                {
                    error!(
                        "Failed to subscribe to all publishers of topic '{}': {}",
                        name,
                        err
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

struct SubscriberInfo<'a> {
    master: &'a Master,
    slave: &'a Slave,
    name: String,
}

impl<'a> Interactor for SubscriberInfo<'a> {
    fn unregister(&mut self) -> Response<()> {
        self.slave.remove_subscription(&self.name);
        self.master.unregister_subscriber(&self.name).map(|_| ())
    }
}

#[derive(Clone)]
pub struct Service<'a> {
    _raii: Arc<InteractorRaii<ServiceInfo<'a>>>,
}

impl<'a> Service<'a> {
    pub(crate) fn new<T, F>(
        master: &'a Master,
        slave: &'a mut Slave,
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

        if let Err(err) = master.register_service(name, &info.api) {
            info.unregister()?;
            Err(err.into())
        } else {
            Ok(Self { _raii: Arc::new(InteractorRaii::new(info)) })
        }
    }
}

struct ServiceInfo<'a> {
    master: &'a Master,
    slave: &'a Slave,
    name: String,
    api: String,
}

impl<'a> Interactor for ServiceInfo<'a> {
    fn unregister(&mut self) -> Response<()> {
        self.slave.remove_service(&self.name);
        self.master.unregister_service(&self.name, &self.api).map(
            |_| (),
        )
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
