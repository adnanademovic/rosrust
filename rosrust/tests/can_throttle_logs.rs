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

    let mut received_messages_counter = 0;

    let rate = rosrust::rate(1.0);
    for _ in 0..10 {
        for item in rx.try_iter() {
            println!("Received message at level {}: {}", item.0, item.1);
            received_messages_counter += 1;
        }

        rosrust::ros_info_throttle!(5.0, "info message");
        rate.sleep();
    }

    if received_messages_counter == 0 {
        panic!("Failed to receive data on /rosout_agg");
    } else if received_messages_counter > 2 {
        panic!(
            "Received {} messages, not throttled",
            received_messages_counter
        );
    }
}
