use crate::api::raii::{Publisher, Service, Subscriber};
use crate::api::resolve::get_unused_args;
use crate::api::{Parameter, Rate, Ros, SystemState, Topic};
use crate::error::{Result, ResultExt};
use crate::rosxmlrpc::Response;
use crate::tcpros::{Client, Message, ServicePair, ServiceResult};
use crate::time::{Duration, Time};
use ctrlc;
use lazy_static::lazy_static;
use log::info;
use std::sync::Mutex;
use std::time;

lazy_static! {
    static ref ROS: Mutex<Option<Ros>> = Mutex::new(None);
}

#[inline]
pub fn init(name: &str) {
    try_init(name).unwrap();
}

pub fn try_init(name: &str) -> Result<()> {
    let mut ros = ROS.lock().expect(LOCK_FAIL);
    if ros.is_some() {
        bail!(INITIALIZED);
    }
    let client = Ros::new(name)?;
    let shutdown_sender = client.shutdown_sender();
    ctrlc::set_handler(move || {
        if shutdown_sender.send(()).is_err() {
            info!("ROS client is already down");
        }
    })
    .chain_err(|| CTRLC_FAIL)?;
    *ros = Some(client);
    Ok(())
}

macro_rules! ros {
    () => {
        ROS.lock().expect(LOCK_FAIL).as_mut().expect(UNINITIALIZED)
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
pub fn sleep(d: Duration) {
    ros!().sleep(d)
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
pub fn shutdown() -> bool {
    ros!().shutdown_sender().send(()).is_ok()
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
    F: Fn(T) -> () + Send + 'static,
{
    ros!().subscribe::<T, F>(topic, queue_size, callback)
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

static CTRLC_FAIL: &str = "Failed to override SIGINT functionality.";
static LOCK_FAIL: &str = "Failed to acquire lock on ROS instance.";
static UNINITIALIZED: &str = "ROS uninitialized. Please run ros::init(name) first!";
static INITIALIZED: &str = "ROS initialized multiple times through ros::init.";
