extern crate env_logger;
#[macro_use]
extern crate rosrust;
#[macro_use]
extern crate rosrust_codegen;

use rosrust::Ros;

rosmsg_include!();

fn main() {
    env_logger::init().unwrap();

    let mut ros = Ros::new("talker").unwrap();
    let mut chatter_pub = ros.publish("chatter").unwrap();

    let mut count = 0;

    let mut rate = ros.rate(10.0);
    while ros.is_ok() {
        let mut msg = msg::std_msgs::String::default();
        msg.data = format!("hello world {}", count);

        ros_info!(ros, msg.data);

        chatter_pub.send(msg).unwrap();

        rate.sleep();

        count += 1;
    }
}
