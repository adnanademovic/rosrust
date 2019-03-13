use crossbeam::channel::unbounded;
use rosrust;
use std::collections::BTreeSet;
use std::process::Command;

mod util;

mod msg {
    rosrust::rosmsg_include!(rosgraph_msgs / Log);
}

#[test]
fn can_read_log_from_rosout_for_rosrust() {
    let _roscore = util::run_roscore_for(util::Language::Multi, util::Feature::Log);
    let _publisher_cpp = util::ChildProcessTerminator::spawn(
        Command::new("rosrun")
            .arg("roscpp_tutorials")
            .arg("talker")
            .arg("__name:=talker_cpp"),
    );
    let _publisher_py = util::ChildProcessTerminator::spawn(
        Command::new("rosrun")
            .arg("rospy_tutorials")
            .arg("talker")
            .arg("__name:=talker_py"),
    );
    let _publisher_rust = util::ChildProcessTerminator::spawn_example(
        "../examples/pubsub",
        Command::new("cargo")
            .arg("run")
            .arg("--bin")
            .arg("publisher")
            .arg("__name:=talker_rust"),
    );

    rosrust::init("rosout_agg_listener");

    let (tx, rx) = unbounded();

    let _subscriber =
        rosrust::subscribe::<msg::rosgraph_msgs::Log, _>("/rosout_agg", 100, move |data| {
            tx.send((data.level, data.name, data.msg)).unwrap();
        })
        .unwrap();

    let mut rate = rosrust::rate(1.0);

    let mut expected_messages = BTreeSet::new();
    expected_messages.insert("/talker_cpp");
    expected_messages.insert("/talker_py");
    expected_messages.insert("/talker_rust");

    for _ in 0..10 {
        for (level, name, message) in rx.try_iter() {
            println!(
                "Received message from {} at level {}: {}",
                name, level, message
            );
            if level == 2 && message.contains("hello world") {
                expected_messages.remove(name.as_str());
            }
        }
        if expected_messages.is_empty() {
            return;
        }
        rate.sleep();
    }

    panic!("Failed to receive data on /rosout_agg");
}
