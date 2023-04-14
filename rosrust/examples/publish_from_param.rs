use std::time::Instant;

mod msg {
    rosrust::rosmsg_include!(std_msgs / String);
}

fn main() {
    env_logger::init();

    // Initialize node
    rosrust::init("talker");

    // Create publisher
    let chatter_pub = rosrust::publish("chatter", 2).unwrap();
    chatter_pub.wait_for_subscribers(None).unwrap();

    let message = rosrust::param("~message").unwrap();

    let log_names = rosrust::param("~log_names").unwrap().get().unwrap_or(false);

    // Create object that maintains 10Hz between sleep requests
    let rate = rosrust::rate(10.0);

    // Breaks when a shutdown signal is sent
    while rosrust::is_ok() {
        let t = Instant::now();
        let message = message.get_raw().unwrap_or(xml_rpc::Value::Bool(false));
        let message_fetch_time = t.elapsed();
        // Create string message
        let msg = msg::std_msgs::String {
            data: format!("{:?} from rosrust in {:?}", message, message_fetch_time),
        };

        // Log event
        rosrust::ros_info!("Publishing: {}", msg.data);

        // Send string message to topic via publisher
        chatter_pub.send(msg).unwrap();

        if log_names {
            rosrust::ros_info!("Subscriber names: {:?}", chatter_pub.subscriber_names());
        }

        // Sleep to maintain 10Hz rate
        rate.sleep();
    }
}
