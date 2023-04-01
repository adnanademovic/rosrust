use crossbeam::channel::unbounded;
use std::collections::BTreeSet;

mod util;

mod msg {
    rosrust::rosmsg_include!(rosgraph_msgs / Log);
}

#[test]
fn can_read_log_from_rosout() {
    let _roscore = util::run_roscore_for(util::TestVariant::CanReadLogFromRosout);

    rosrust::init("rosout_agg_listener");

    let (tx, rx) = unbounded();

    let _subscriber =
        rosrust::subscribe::<msg::rosgraph_msgs::Log, _>("/rosout_agg", 100, move |data| {
            tx.send((data.level, data.msg)).unwrap();
        })
        .unwrap();

    let rate = rosrust::rate(1.0);

    let mut expected_messages = BTreeSet::new();
    expected_messages.insert((1, "debug message".to_owned()));
    expected_messages.insert((2, "info message".to_owned()));
    expected_messages.insert((4, "warn message".to_owned()));
    expected_messages.insert((8, "err message".to_owned()));
    expected_messages.insert((16, "fatal message".to_owned()));

    for _ in 0..10 {
        for item in rx.try_iter() {
            println!("Received message at level {}: {}", item.0, item.1);
            expected_messages.remove(&item);
        }

        if expected_messages.is_empty() {
            return;
        }

        rosrust::ros_debug!("debug message");
        rosrust::ros_info!("info message");
        rosrust::ros_warn!("warn message");
        rosrust::ros_err!("err message");
        rosrust::ros_fatal!("fatal message");
        rate.sleep();
    }

    panic!("Failed to receive data on /rosout_agg");
}
