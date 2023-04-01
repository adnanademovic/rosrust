use rosrust::RawMessageDescription;

mod msg {
    rosrust::rosmsg_include!(std_msgs / String);
}

fn main() {
    env_logger::init();

    // Initialize node
    rosrust::init("talker");

    // Create publisher
    let chatter_pub = rosrust::publish_with_description(
        "chatter",
        2,
        RawMessageDescription {
            msg_definition: "string data\n".into(),
            md5sum: "992ce8a1687cec8c8bd883ec73ca41d1".into(),
            msg_type: "std_msgs/String".into(),
        },
    )
    .unwrap();
    chatter_pub.wait_for_subscribers(None).unwrap();

    let log_names = rosrust::param("~log_names").unwrap().get().unwrap_or(false);

    let mut count = 0;

    // Create object that maintains 10Hz between sleep requests
    let rate = rosrust::rate(10.0);

    // Breaks when a shutdown signal is sent
    while rosrust::is_ok() {
        // Create string message
        let msg = rosrust::RawMessage(vec![
            27,
            0,
            0,
            0,
            104,
            101,
            108,
            108,
            111,
            32,
            119,
            111,
            114,
            108,
            100,
            32,
            102,
            114,
            111,
            109,
            32,
            114,
            111,
            115,
            114,
            117,
            115,
            116,
            32,
            48 + (count / 10) % 10,
            48 + count % 10,
        ]);

        // Log event
        rosrust::ros_info!("Publishing: {:?}", msg.0);

        // Send string message to topic via publisher
        chatter_pub.send(msg).unwrap();

        if log_names {
            rosrust::ros_info!("Subscriber names: {:?}", chatter_pub.subscriber_names());
        }

        // Sleep to maintain 10Hz rate
        rate.sleep();

        count += 1;
    }
}
