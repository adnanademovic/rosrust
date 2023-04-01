use rosrust::{self, ros_info};

fn main() {
    env_logger::init();

    // Initialize node
    rosrust::init("talker");

    // Create publisher
    let mut chatter_pub_latched = rosrust::publish("chatter", 2).unwrap();
    let chatter_pub_unlatched = rosrust::publish("chatter", 2).unwrap();
    chatter_pub_latched.set_latching(true);

    chatter_pub_latched.wait_for_subscribers(None).unwrap();
    chatter_pub_unlatched.wait_for_subscribers(None).unwrap();

    let mut msg = rosrust_msg::std_msgs::String {
        data: String::from("hello world latched"),
    };

    // Log event
    ros_info!("Publishing: {}", msg.data);

    // Send string message to topic via publisher
    chatter_pub_latched.send(msg.clone()).unwrap();

    msg.data = String::from("hello world unlatched");

    // Log event
    ros_info!("Publishing: {}", msg.data);

    // Send string message to topic via publisher, without latching it
    chatter_pub_unlatched.send(msg).unwrap();

    rosrust::spin();
}
