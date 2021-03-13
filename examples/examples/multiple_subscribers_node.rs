use env_logger;
use rosrust;
use rosrust::{sleep, Duration, RawMessage};

fn main() {
    env_logger::init();

    // Initialize node
    rosrust::init("listener");

    // Create subscriber
    // The subscriber is stopped when all the returned objects are destroyed
    let subscriber_info_1 = rosrust::subscribe_with_ids_and_headers(
        "chatter",
        2,
        |v: RawMessage, id| {
            // Callback for handling received messages
            rosrust::ros_info!("Received 1 from {}: {:?}", id, v.0);
        },
        |headers| {
            // Callback for handling received messages
            rosrust::ros_info!("Connected 1 to: {:#?}", headers);
        },
    )
    .unwrap();

    sleep(Duration::from_seconds(2));

    // Create subscriber
    // The subscriber is stopped when all the returned objects are destroyed
    let subscriber_info_2 = rosrust::subscribe_with_ids_and_headers(
        "chatter",
        5,
        |v: rosrust_msg::std_msgs::String, id| {
            // Callback for handling received messages
            rosrust::ros_info!("Received 2 from {}: {}", id, v.data);
        },
        |headers| {
            // Callback for handling received messages
            rosrust::ros_info!("Connected 2 to: {:#?}", headers);
        },
    )
    .unwrap();

    let log_names = rosrust::param("~log_names").unwrap().get().unwrap_or(false);

    if log_names {
        let rate = rosrust::rate(1.0);
        while rosrust::is_ok() {
            rosrust::ros_info!("Publisher uris 1: {:?}", subscriber_info_1.publisher_uris());
            rosrust::ros_info!("Publisher uris 2: {:?}", subscriber_info_2.publisher_uris());
            rate.sleep();
        }
    } else {
        // Block the thread until a shutdown signal is received
        rosrust::spin();
    }
}
