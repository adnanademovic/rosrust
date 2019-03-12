use crossbeam::channel::unbounded;
use rosrust;
use std::process::Command;

mod util;

mod msg {
    rosrust::rosmsg_include!(std_msgs / String);
}

#[test]
fn subscriber_to_roscpp_publisher() {
    let _roscore = util::run_roscore_for(util::Language::Cpp, util::Feature::Subscriber);
    let _publisher = util::ChildProcessTerminator::spawn(
        Command::new("rosrun").arg("roscpp_tutorials").arg("talker"),
    );

    let (tx, rx) = unbounded();

    rosrust::init("hello_world_listener");
    let _subscriber = rosrust::subscribe::<msg::std_msgs::String, _>("chatter", 100, move |data| {
        tx.send(data.data).unwrap();
    })
    .unwrap();

    util::test_subscriber(rx, r"hello world (\d+)", true, 20);
}
