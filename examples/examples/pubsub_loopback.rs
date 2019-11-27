use env_logger;
use rosrust;

fn main() {
    env_logger::init();

    // Initialize node
    rosrust::init("talker_listener");

    // Create subscriber
    // The subscriber is stopped when the returned object is destroyed
    let _subscriber_raii =
        rosrust::subscribe("chatter", 200, |v: rosrust_msg::std_msgs::String| {
            // Callback for handling received messages
            rosrust::ros_info!("Received: {}", v.data);
        })
        .unwrap();

    // Create publisher
    let chatter_pub = rosrust::publish("chatter", 200).unwrap();

    let mut count = 0;

    // Create object that maintains 10Hz between sleep requests
    let rate = rosrust::rate(10.0);

    // Breaks when a shutdown signal is sent
    while rosrust::is_ok() {
        // Create string message
        let mut msg = rosrust_msg::std_msgs::String::default();
        msg.data = format!("hello world from rosrust {}", count);

        // Log event
        rosrust::ros_info!("Publishing: {}", msg.data);

        // Send string message to topic via publisher
        chatter_pub.send(msg).unwrap();

        // Sleep to maintain 10Hz rate
        rate.sleep();

        count += 1;
    }
}
