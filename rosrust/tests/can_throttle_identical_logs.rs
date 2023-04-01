use crossbeam::channel::unbounded;
use ros_message::Duration;

mod util;

mod msg {
    rosrust::rosmsg_include!(rosgraph_msgs / Log);
}

#[test]
fn can_throttle_identical_logs() {
    let _roscore = util::run_roscore_for(util::TestVariant::CanThrottleIdenticalLogs);

    rosrust::init("rosout_agg_listener");

    let (tx, rx) = unbounded();

    let _subscriber =
        rosrust::subscribe::<msg::rosgraph_msgs::Log, _>("/rosout_agg", 100, move |data| {
            tx.send((data.level, data.msg)).unwrap();
        })
        .unwrap();

    rosrust::sleep(Duration::from_seconds(1));

    let mut received_messages_counter = 0;

    let rate = rosrust::rate(2.0);
    for i in 0..20 {
        for item in rx.try_iter() {
            println!("Received message at level {}: {}", item.0, item.1);
            received_messages_counter += 1;
        }

        let prefix = if i == 4 { "second" } else { "first" };
        rosrust::ros_info_throttle_identical!(3.0, "{} message", prefix);
        rate.sleep();
    }

    if received_messages_counter == 0 {
        panic!("Failed to receive data on /rosout_agg");
    } else if received_messages_counter != 5 {
        panic!(
            "Received {} messages, but should be 5",
            received_messages_counter
        );
    }
}
