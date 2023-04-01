use crossbeam::channel::unbounded;
use std::process::Command;

mod util;

mod msg {
    rosrust::rosmsg_include!(std_msgs / String);
}

#[test]
fn subscriber_to_multiple_publishers() {
    let _roscore = util::run_roscore_for(util::TestVariant::SubscriberToMultiplePublishers);
    let _publisher_rostopic = util::ChildProcessTerminator::spawn(
        Command::new("rostopic")
            .arg("pub")
            .arg("-r")
            .arg("5")
            .arg("chatter")
            .arg("std_msgs/String")
            .arg("hello world from rostopic"),
    );
    let _publisher_roscpp = util::ChildProcessTerminator::spawn(
        Command::new("rosrun")
            .arg("roscpp_tutorials")
            .arg("talker")
            .arg("__name:=talker_cpp"),
    );
    let _publisher_rospy = util::ChildProcessTerminator::spawn(
        Command::new("rosrun")
            .arg("rospy_tutorials")
            .arg("talker")
            .arg("__name:=talker_py"),
    );
    let _publisher = util::ChildProcessTerminator::spawn_example(
        Command::new("cargo")
            .arg("run")
            .arg("--example")
            .arg("publisher")
            .arg("__name:=talker_rust"),
    );

    let (tx, rx) = unbounded();

    rosrust::init("hello_world_listener");
    let subscriber = rosrust::subscribe::<msg::std_msgs::String, _>("chatter", 100, move |data| {
        tx.send(data.data).unwrap();
    })
    .unwrap();

    println!("Checking roscpp publisher");
    util::test_subscriber_detailed(rx.clone(), r"^hello world (\d+)$", true, 10, false);
    println!("Checking rospy publisher");
    util::test_subscriber_detailed(rx.clone(), r"^hello world (\d+\.\d+)$", true, 10, false);
    println!("Checking rosrust publisher");
    util::test_subscriber_detailed(
        rx.clone(),
        r"^hello world from rosrust (\d+)$",
        true,
        10,
        false,
    );
    println!("Checking rostopic publisher");
    util::test_subscriber_detailed(rx, r"^hello world from rostopic$", false, 10, false);

    assert_eq!(subscriber.publisher_count(), 4);
}
