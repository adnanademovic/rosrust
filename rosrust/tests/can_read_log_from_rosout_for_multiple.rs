use crossbeam::channel::unbounded;
use std::collections::BTreeSet;
use std::process::Command;

mod util;

mod msg {
    rosrust::rosmsg_include!(rosgraph_msgs / Log);
}

#[test]
fn can_read_log_from_rosout_for_multiple() {
    let _roscore = util::run_roscore_for(util::TestVariant::CanReadLogFromRosoutForMultiple);
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
        Command::new("cargo")
            .arg("run")
            .arg("--example")
            .arg("publisher")
            .arg("__name:=talker_rust"),
    );

    rosrust::init("rosout_agg_listener");

    let (tx, rx) = unbounded();
    let tx_agg = tx.clone();

    let _subscriber =
        rosrust::subscribe::<msg::rosgraph_msgs::Log, _>("/rosout", 100, move |data| {
            tx.send((false, data.level, data.name, data.msg)).unwrap();
        })
        .unwrap();

    let _subscriber_agg =
        rosrust::subscribe::<msg::rosgraph_msgs::Log, _>("/rosout_agg", 100, move |data| {
            tx_agg
                .send((true, data.level, data.name, data.msg))
                .unwrap();
        })
        .unwrap();

    let rate = rosrust::rate(1.0);

    let mut expected_messages = BTreeSet::new();
    expected_messages.insert((true, "/talker_cpp".to_owned()));
    expected_messages.insert((false, "/talker_cpp".to_owned()));
    expected_messages.insert((true, "/talker_py".to_owned()));
    expected_messages.insert((false, "/talker_py".to_owned()));
    expected_messages.insert((true, "/talker_rust".to_owned()));
    expected_messages.insert((false, "/talker_rust".to_owned()));

    for _ in 0..10 {
        for (aggregated, level, name, message) in rx.try_iter() {
            println!(
                "Received {}message from {} at level {}: {}",
                if aggregated { "aggregated " } else { "" },
                name,
                level,
                message,
            );
            if level == 2 && message.contains("hello world") {
                expected_messages.remove(&(aggregated, name));
            }
        }
        if expected_messages.is_empty() {
            return;
        }
        rate.sleep();
    }

    let cases = expected_messages
        .iter()
        .map(|(aggregated, name)| {
            format!(
                "{} on {}",
                name,
                if *aggregated {
                    "/rosout_agg"
                } else {
                    "/rosout"
                },
            )
        })
        .collect::<Vec<String>>();

    panic!("Failed to receive data from: {:?}", cases);
}
