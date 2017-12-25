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

    let ros = Arc::new(Mutex::new(Ros::new("listener").unwrap()));
    let ros_thread = Arc::clone(&ros);

    // The subscriber is stopped when the returned object is destroyed
    let _subscriber_raii = ros.lock()
        .unwrap()
        .subscribe("chatter", move |v: msg::std_msgs::String| {
            let mut ros = ros_thread.lock().unwrap();
            println!("{}", v.data);
            ros_info!(ros, v.data);
        })
        .unwrap();

    let spin = { ros.lock().unwrap().spin() };
}
