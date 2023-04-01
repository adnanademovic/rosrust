use crossbeam::channel::unbounded;
use std::process::Command;

mod util;

mod msg {
    rosrust::rosmsg_include!(std_msgs / String, std_msgs / String);
}

#[test]
fn publisher_to_relayed_subscriber() {
    let _roscore = util::run_roscore_for(util::TestVariant::PublisherToRelayedSubscriber);
    let _subscriber = util::ChildProcessTerminator::spawn(
        Command::new("rosrun")
            .arg("topic_tools")
            .arg("relay")
            .arg("/chatter_pub")
            .arg("/chatter_sub"),
    );

    rosrust::init("hello_world_talker_listener");

    let (tx, rx) = unbounded();

    let _log_subscriber =
        rosrust::subscribe::<msg::std_msgs::String, _>("chatter_sub", 100, move |data| {
            tx.send((2, data.data)).unwrap();
        })
        .unwrap();

    let publisher = rosrust::publish::<msg::std_msgs::String>("chatter_pub", 100).unwrap();
    publisher.wait_for_subscribers(None).unwrap();

    let message = msg::std_msgs::String {
        data: "hello world".into(),
    };

    util::test_publisher(&publisher, &message, &rx, r"^hello world", 50);

    assert_eq!(publisher.subscriber_count(), 1);
}
