extern crate env_logger;
#[macro_use]
extern crate rosrust;
#[macro_use]
extern crate rosrust_codegen;
#[macro_use]
extern crate serde_derive;

mod msg;

fn main() {
    env_logger::init();

    // Initialize node
    rosrust::init("listener");

    // Create subscriber
    // The subscriber is stopped when the returned object is destroyed
    let _subscriber_raii = rosrust::subscribe("chatter", |v: msg::std_msgs::String| {
        // Callback for handling received messages
        ros_info!("Received: {}", v.data);
    }).unwrap();

    // Block the thread until a shutdown signal is received
    rosrust::spin();
}
