extern crate env_logger;
extern crate rosrust;
#[macro_use]
extern crate rosrust_codegen;

use rosrust::Ros;

rosmsg_include!();

fn main() {
    env_logger::init().unwrap();

    let mut ros = Ros::new("listener").unwrap();

    // The subscriber is stopped when the returned object is destroyed
    let _subscriber_raii =
        ros.subscribe("chatter", |v: msg::std_msgs::String| println!("{}", v.data))
            .unwrap();

    ros.spin();
}
