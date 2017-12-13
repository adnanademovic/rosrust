extern crate env_logger;
#[macro_use]
extern crate rosrust;

use rosrust::Ros;
use std::{thread, time};

rosmsg_include!();

fn main() {
    env_logger::init().unwrap();

    let mut ros = Ros::new("talker").unwrap();
    let mut chatter_pub = ros.publish("chatter").unwrap();

    let mut count = 0;

    loop {
        let mut msg = msg::std_msgs::String::new();
        msg.data = format!("hello world {}", count);

        chatter_pub.send(msg).unwrap();

        thread::sleep(time::Duration::from_millis(100));

        count += 1;
    }
}
