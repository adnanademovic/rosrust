use env_logger;
use rosrust;
use std::collections::HashMap;

mod msg {
    rosrust::rosmsg_include!(std_msgs / String);
}

fn main() {
    env_logger::init();

    // Initialize node
    rosrust::init("listener");

    // Create subscriber
    // The subscriber is stopped when the returned object is destroyed
    let subscriber_info = rosrust::subscribe_with_ids_and_headers(
        "chatter",
        2,
        |v: rosrust::RawSubMessage, id: &str| {
            // Callback for handling received messages
            rosrust::ros_info!("Received from '{}': {:?}", id, v.0);
        },
        |headers: HashMap<String, String>| {
            rosrust::ros_info!("Connected to publisher with headers: {:#?}", headers);
        },
    )
    .unwrap();

    let log_names = rosrust::param("~log_names").unwrap().get().unwrap_or(false);

    if log_names {
        let rate = rosrust::rate(1.0);
        while rosrust::is_ok() {
            rosrust::ros_info!("Publisher uris: {:?}", subscriber_info.publisher_uris());
            rate.sleep();
        }
    } else {
        // Block the thread until a shutdown signal is received
        rosrust::spin();
    }
}
