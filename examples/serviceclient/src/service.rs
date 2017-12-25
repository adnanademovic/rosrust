extern crate env_logger;
#[macro_use]
extern crate rosrust;
#[macro_use]
extern crate rosrust_codegen;

use rosrust::Ros;
use std::sync::{Arc, Mutex};

rosmsg_include!();

fn main() {
    env_logger::init().unwrap();

    let ros = Arc::new(Mutex::new(Ros::new("add_two_ints_server").unwrap()));

    let ros_thread = Arc::clone(&ros);

    // The service is stopped when the returned object is destroyed
    let _service_raii = ros.lock()
        .unwrap()
        .service::<msg::roscpp_tutorials::TwoInts, _>("add_two_ints", move |req| {
            let mut ros = ros_thread.lock().unwrap();
            let sum = req.a + req.b;
            ros_info!(ros, format!("{} + {} = {}", req.a, req.b, sum));
            Ok(msg::roscpp_tutorials::TwoIntsRes { sum })
        })
        .unwrap();

    let spin = { ros.lock().unwrap().spin() };
}
