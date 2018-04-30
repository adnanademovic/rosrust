extern crate env_logger;
#[macro_use]
extern crate rosrust;
#[macro_use]
extern crate rosrust_codegen;

rosmsg_include!();

fn main() {
    env_logger::init();

    // Initialize node
    rosrust::init("talker");

    // Create publisher
    let mut chatter_pub = rosrust::publish("chatter").unwrap();

    let mut msg = msg::std_msgs::String::default();
    msg.data = String::from("hello world");

    // Log event
    ros_info!("Publishing: {}", msg.data);

    // Send string message to topic via publisher
    chatter_pub.send(msg).unwrap();

    rosrust::spin();
}
