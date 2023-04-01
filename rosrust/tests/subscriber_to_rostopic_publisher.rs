use crossbeam::channel::unbounded;
use std::process::Command;

mod util;

mod msg {
    rosrust::rosmsg_include!(std_msgs / String);
}

#[test]
fn subscriber_to_rostopic_publisher() {
    let _roscore = util::run_roscore_for(util::TestVariant::SubscriberToRostopicPublisher);
    let _publisher = util::ChildProcessTerminator::spawn(
        Command::new("rostopic")
            .arg("pub")
            .arg("-r")
            .arg("100")
            .arg("chatter")
            .arg("std_msgs/String")
            .arg("hello world from rostopic"),
    );

    let (tx, rx) = unbounded();

    rosrust::init("hello_world_listener");
    let subscriber = rosrust::subscribe::<msg::std_msgs::String, _>("chatter", 100, move |data| {
        tx.send(data.data).unwrap();
    })
    .unwrap();

    util::test_subscriber(rx, r"hello world from rostopic", false, 200);

    assert_eq!(subscriber.publisher_count(), 1);
}
