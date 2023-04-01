use crossbeam::channel::unbounded;
use ros_message::Duration;

mod util;

mod msg {
    rosrust::rosmsg_include!(rosgraph_msgs / Log);
}

#[test]
fn can_log_once() {
    let _roscore = util::run_roscore_for(util::TestVariant::CanLogOnce);

    rosrust::init("rosout_agg_listener");

    let (tx, rx) = unbounded();

    let _subscriber =
        rosrust::subscribe::<msg::rosgraph_msgs::Log, _>("/rosout_agg", 100, move |data| {
            tx.send((data.level, data.msg)).unwrap();
        })
        .unwrap();

    rosrust::sleep(Duration::from_seconds(1));

    let mut received_messages_counter = 0;

    let rate = rosrust::rate(1.0);
    for _ in 0..10 {
        for item in rx.try_iter() {
            println!("Received message at level {}: {}", item.0, item.1);
            received_messages_counter += 1;
        }

        rosrust::ros_info_once!("info message");
        rate.sleep();
    }

    if received_messages_counter == 0 {
        panic!("Failed to receive data on /rosout_agg");
    } else if received_messages_counter > 1 {
        panic!(
            "Received {} messages, not printed once",
            received_messages_counter
        );
    }
}
