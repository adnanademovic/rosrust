use crossbeam::channel::unbounded;

mod util;

mod msg {
    rosrust::rosmsg_include!(rosgraph_msgs / Log);
}

#[test]
fn can_throttle_logs() {
    let _roscore = util::run_roscore_for(util::TestVariant::CanThrottleLogs);

    rosrust::init("rosout_agg_listener");

    let (tx, rx) = unbounded();

    let _subscriber =
        rosrust::subscribe::<msg::rosgraph_msgs::Log, _>("/rosout_agg", 100, move |data| {
            tx.send((data.level, data.msg)).unwrap();
        })
        .unwrap();

    // Wait for the pub/sub connection to be established
    rosrust::sleep(rosrust::Duration::from_seconds(1));

    let mut received_messages_counter = 0;

    let rate = rosrust::rate(10.0);
    let start_time = rosrust::now();
    for i in 0..11 {
        // Add a gap between t = 0.3s and t = 0.7s
        // Logs should be produced at:
        // t=0s, t=0.2s, t=0.7s, t=0.9s
        if i < 4 || i > 6 {
            rosrust::ros_info_throttle!(0.2, "[{}] info message", rosrust::now() - start_time);
        }
        rate.sleep();
    }

    for item in rx.try_iter() {
        println!("Received message at level {}: {}", item.0, item.1);
        received_messages_counter += 1;
    }

    if received_messages_counter == 0 {
        panic!("Failed to receive data on /rosout_agg");
    } else if received_messages_counter != 4 {
        panic!(
            "Received {} messages, not throttled properly",
            received_messages_counter
        );
    }
}
