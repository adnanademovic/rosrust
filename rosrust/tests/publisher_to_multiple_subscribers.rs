use crossbeam::channel::unbounded;
use std::process::Command;

mod util;

mod msg {
    rosrust::rosmsg_include!(std_msgs / String, rosgraph_msgs / Log);
}

#[test]
fn publisher_to_multiple_subscribers() {
    let _roscore = util::run_roscore_for(util::TestVariant::PublisherToMultipleSubscribers);
    let _subscriber_cpp = util::ChildProcessTerminator::spawn(
        Command::new("rosrun")
            .arg("roscpp_tutorials")
            .arg("listener")
            .arg("__name:=listener_cpp"),
    );
    let _subscriber_py = util::ChildProcessTerminator::spawn(
        Command::new("rosrun")
            .arg("rospy_tutorials")
            .arg("listener"),
    );
    let _subscriber_rust = util::ChildProcessTerminator::spawn_example(
        Command::new("cargo")
            .arg("run")
            .arg("--example")
            .arg("subscriber"),
    );

    rosrust::init("hello_world_talker");

    let (tx, rx) = unbounded();

    let _log_subscriber =
        rosrust::subscribe::<msg::rosgraph_msgs::Log, _>("/rosout_agg", 100, move |data| {
            tx.send((data.level, data.msg)).unwrap();
        })
        .unwrap();

    let publisher = rosrust::publish::<msg::std_msgs::String>("chatter", 100).unwrap();
    publisher.wait_for_subscribers(None).unwrap();

    let message = msg::std_msgs::String {
        data: "hello world".into(),
    };

    println!("Checking roscpp subscriber");
    util::test_publisher(&publisher, &message, &rx, r"^I heard: \[hello world\]$", 50);
    println!("Checking rospy subscriber");
    util::test_publisher(&publisher, &message, &rx, r"I heard hello world$", 50);
    println!("Checking rosrust subscriber");
    util::test_publisher(&publisher, &message, &rx, r"^Received: hello world$", 50);

    assert_eq!(publisher.subscriber_count(), 3);
}
