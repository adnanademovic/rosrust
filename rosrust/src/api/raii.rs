use super::clock::Clock;
use super::error::Result;
use super::master::Master;
use super::slave::Slave;
use crate::api::SystemState;
use crate::error::ErrorKind;
use crate::rosxmlrpc::Response;
use crate::tcpros::{Message, PublisherStream, ServicePair, ServiceResult};
use crate::{RawMessageDescription, SubscriptionHandler};
use log::error;
use std::sync::atomic::AtomicUsize;
use std::sync::Arc;

#[derive(Clone)]
pub struct Publisher<T: Message> {
    clock: Arc<dyn Clock>,
    seq: Arc<AtomicUsize>,
    stream: PublisherStream<T>,
    raii: Arc<InteractorRaii<PublisherInfo>>,
}

impl<T: Message> Publisher<T> {
    pub(crate) fn new(
        master: Arc<Master>,
        slave: Arc<Slave>,
        clock: Arc<dyn Clock>,
        hostname: &str,
        name: &str,
        queue_size: usize,
        message_description: Option<RawMessageDescription>,
    ) -> Result<Self> {
        let message_description =
            message_description.unwrap_or_else(RawMessageDescription::from_message::<T>);
        let stream =
            slave.add_publication::<T>(hostname, name, queue_size, message_description.clone())?;

        let raii = Arc::new(InteractorRaii::new(PublisherInfo {
            master,
            slave,
            name: name.into(),
        }));

        raii.interactor
            .master
            .register_publisher(name, &message_description.msg_type)
            .map_err(|err| {
                error!("Failed to register publisher for topic '{}': {}", name, err);
                err
            })?;

        Ok(Self {
            stream,
            clock,
            seq: Arc::new(AtomicUsize::new(0)),
            raii,
        })
    }

    #[inline]
    pub fn subscriber_count(&self) -> usize {
        self.stream.subscriber_count()
    }

    #[inline]
    pub fn subscriber_names(&self) -> Vec<String> {
        self.stream.subscriber_names()
    }

    #[inline]
    pub fn set_latching(&mut self, latching: bool) {
        self.stream.set_latching(latching);
    }

    #[inline]
    pub fn set_queue_size(&mut self, queue_size: usize) {
        self.stream.set_queue_size(queue_size);
    }

    /// Wait until all the subscribers reported by rosmaster have connected
    #[inline]
    pub fn wait_for_subscribers(&self, timeout: Option<std::time::Duration>) -> Result<()> {
        let timeout = timeout.map(|v| std::time::Instant::now() + v);
        let iteration_time = std::time::Duration::from_millis(50);
        loop {
            let system_state: SystemState = self.raii.interactor.master.get_system_state()?.into();
            let mut master_subs = system_state
                .subscribers
                .into_iter()
                .find(|v| v.name == self.raii.interactor.name)
                .map(|v| v.connections)
                .unwrap_or_default();
            master_subs.sort();
            let mut local_subs = self.subscriber_names();
            local_subs.sort();
            if master_subs == local_subs {
                return Ok(());
            }
            let now = std::time::Instant::now();
            let mut wait_time = iteration_time;
            if let Some(timeout) = &timeout {
                let time_left = timeout
                    .checked_duration_since(now)
                    .ok_or(ErrorKind::TimeoutError)?;
                wait_time = wait_time.min(time_left);
            }
            std::thread::sleep(wait_time);
        }
    }

    #[inline]
    pub fn send(&self, mut message: T) -> Result<()> {
        message.set_header(&self.clock, &self.seq);
        self.stream.send(&message).map_err(Into::into)
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
    info: Arc<InteractorRaii<SubscriberInfo>>,
}

impl Subscriber {
    pub(crate) fn new<T, H>(
        master: Arc<Master>,
        slave: Arc<Slave>,
        name: &str,
        queue_size: usize,
        handler: H,
    ) -> Result<Self>
    where
        T: Message,
        H: SubscriptionHandler<T>,
    {
        let id = slave.add_subscription::<T, H>(name, queue_size, handler)?;

        let info = Arc::new(InteractorRaii::new(SubscriberInfo {
            master,
            slave,
            name: name.into(),
            id,
        }));

        let publishers = info
            .interactor
            .master
            .register_subscriber(name, &T::msg_type())?;

        if let Err(err) = info
            .interactor
            .slave
            .add_publishers_to_subscription(name, publishers.into_iter())
        {
            error!(
                "Failed to subscribe to all publishers of topic '{}': {}",
                name, err
            );
        }

        Ok(Self { info })
    }

    #[inline]
    pub fn publisher_count(&self) -> usize {
        self.info
            .interactor
            .slave
            .get_publisher_count_of_subscription(&self.info.interactor.name)
    }

    #[inline]
    pub fn publisher_uris(&self) -> Vec<String> {
        self.info
            .interactor
            .slave
            .get_publisher_uris_of_subscription(&self.info.interactor.name)
    }
}

struct SubscriberInfo {
    master: Arc<Master>,
    slave: Arc<Slave>,
    name: String,
    id: usize,
}

impl Interactor for SubscriberInfo {
    fn unregister(&mut self) -> Response<()> {
        self.slave.remove_subscription(&self.name, self.id);
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
        bind_address: &str,
        name: &str,
        handler: F,
    ) -> Result<Self>
    where
        T: ServicePair,
        F: Fn(T::Request) -> ServiceResult<T::Response> + Send + Sync + 'static,
    {
        let api = slave.add_service::<T, F>(hostname, bind_address, name, handler)?;

        let raii = Arc::new(InteractorRaii::new(ServiceInfo {
            master,
            slave,
            api,
            name: name.into(),
        }));

        raii.interactor
            .master
            .register_service(name, &raii.interactor.api)?;
        Ok(Self { _raii: raii })
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
    pub interactor: I,
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
