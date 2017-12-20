extern crate env_logger;
#[macro_use]
extern crate rosrust;

use rosrust::Ros;
use std::{thread, time};

rosmsg_include!();

fn main() {
    env_logger::init().unwrap();

    let mut ros = Ros::new("add_two_ints_server").unwrap();

    // The service is stopped when the returned object is destroyed
    let _service_raii = ros.service::<msg::roscpp_tutorials::TwoInts, _>("add_two_ints", |req| {
        Ok(msg::roscpp_tutorials::TwoIntsRes { sum: req.a + req.b })
    }).unwrap();

    loop {
        thread::sleep(time::Duration::from_secs(100));
    }
}
