use crossbeam::channel::unbounded;
use std::process::Command;

mod util;

mod msg {
    rosrust::rosmsg_include!(std_msgs / String);
}

#[test]
fn subscriber_to_rosrust_publisher() {
    let _roscore = util::run_roscore_for(util::TestVariant::SubscriberToRosrustPublisher);
    let _publisher = util::ChildProcessTerminator::spawn_example(
        Command::new("cargo")
            .arg("run")
            .arg("--example")
            .arg("publisher"),
    );

    let (tx, rx) = unbounded();

    rosrust::init("hello_world_listener");
    let subscriber = rosrust::subscribe::<msg::std_msgs::String, _>("chatter", 100, move |data| {
        tx.send(data.data).unwrap();
    })
    .unwrap();

    util::test_subscriber(rx, r"hello world from rosrust (\d+)", true, 20);

    assert_eq!(subscriber.publisher_count(), 1);
}
