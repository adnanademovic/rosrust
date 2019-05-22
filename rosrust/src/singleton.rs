use crate::api::raii::{Publisher, Service, Subscriber};
use crate::api::resolve::get_unused_args;
use crate::api::{Delay, Parameter, Rate, Ros, SystemState, Topic};
use crate::error::{ErrorKind, Result};
use crate::rosxmlrpc::Response;
use crate::tcpros::{Client, Message, ServicePair, ServiceResult};
use crate::time::{Duration, Time};
use crate::util::FAILED_TO_LOCK;
use crossbeam::sync::ShardedLock;
use ctrlc;
use lazy_static::lazy_static;
use std::time;

lazy_static! {
    static ref ROS: ShardedLock<Option<Ros>> = ShardedLock::new(None);
}

#[inline]
pub fn init(name: &str) {
    try_init(name).expect("ROS init failed!");
}

#[inline]
pub fn try_init(name: &str) -> Result<()> {
    try_init_with_options(name, true)
}

pub fn try_init_with_options(name: &str, capture_sigint: bool) -> Result<()> {
    let mut ros = ROS.write().expect(FAILED_TO_LOCK);
    if ros.is_some() {
        bail!(ErrorKind::MultipleInitialization);
    }
    let client = Ros::new(name)?;
    if capture_sigint {
        let shutdown_sender = client.shutdown_sender();
        ctrlc::set_handler(move || {
            shutdown_sender.shutdown();
        })?;
    }
    *ros = Some(client);
    Ok(())
}

pub fn is_initialized() -> bool {
    ROS.read().expect(FAILED_TO_LOCK).is_some()
}

macro_rules! ros {
    () => {
        ROS.read()
            .expect(FAILED_TO_LOCK)
            .as_ref()
            .expect(UNINITIALIZED)
    };
}

#[inline]
pub fn args() -> Vec<String> {
    get_unused_args()
}

#[inline]
pub fn uri() -> String {
    ros!().uri().into()
}

#[inline]
pub fn name() -> String {
    ros!().name().into()
}

#[inline]
pub fn hostname() -> String {
    ros!().hostname().into()
}

#[inline]
pub fn now() -> Time {
    ros!().now()
}

#[inline]
pub fn delay(d: Duration) -> Delay {
    ros!().delay(d)
}

#[inline]
pub fn sleep(d: Duration) {
    delay(d).sleep();
}

#[inline]
pub fn rate(rate: f64) -> Rate {
    ros!().rate(rate)
}

#[inline]
pub fn is_ok() -> bool {
    ros!().is_ok()
}

#[inline]
pub fn spin() {
    // Spinner drop locks the thread until a kill request is received
    let _spinner = { ros!().spin() };
}

#[inline]
pub fn shutdown() {
    ros!().shutdown_sender().shutdown()
}

#[inline]
pub fn param(name: &str) -> Option<Parameter> {
    ros!().param(name)
}

#[inline]
pub fn parameters() -> Response<Vec<String>> {
    ros!().parameters()
}

#[inline]
pub fn state() -> Response<SystemState> {
    ros!().state()
}

#[inline]
pub fn topics() -> Response<Vec<Topic>> {
    ros!().topics()
}

#[inline]
pub fn client<T: ServicePair>(service: &str) -> Result<Client<T>> {
    ros!().client::<T>(service)
}

#[inline]
pub fn wait_for_service(service: &str, timeout: Option<time::Duration>) -> Result<()> {
    ros!().wait_for_service(service, timeout)
}

#[inline]
pub fn service<T, F>(service: &str, handler: F) -> Result<Service>
where
    T: ServicePair,
    F: Fn(T::Request) -> ServiceResult<T::Response> + Send + Sync + 'static,
{
    ros!().service::<T, F>(service, handler)
}

#[inline]
pub fn subscribe<T, F>(topic: &str, queue_size: usize, callback: F) -> Result<Subscriber>
where
    T: Message,
    F: Fn(T) + Send + 'static,
{
    ros!().subscribe::<T, F>(topic, queue_size, callback)
}

#[inline]
pub fn subscribe_with_ids<T, F>(topic: &str, queue_size: usize, callback: F) -> Result<Subscriber>
where
    T: Message,
    F: Fn(T, &str) + Send + 'static,
{
    ros!().subscribe_with_ids::<T, F>(topic, queue_size, callback)
}

#[inline]
pub fn publish<T>(topic: &str, queue_size: usize) -> Result<Publisher<T>>
where
    T: Message,
{
    ros!().publish::<T>(topic, queue_size)
}

#[inline]
pub fn log(level: i8, msg: String, file: &str, line: u32) {
    ros!().log(level, msg, file, line)
}

static UNINITIALIZED: &str = "ROS uninitialized. Please run ros::init(name) first!";
