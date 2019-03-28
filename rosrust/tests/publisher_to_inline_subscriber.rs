use crossbeam::channel::unbounded;
use rosrust;
use std::process::Command;

mod util;

mod msg {
    rosrust::rosmsg_include!(std_msgs / String, std_msgs / String);
}

#[test]
fn publisher_to_inline_subscriber() {
    let _roscore = util::run_roscore_for(util::Language::None, util::Feature::Publisher);
    let _subscriber = util::ChildProcessTerminator::spawn_example(
        "../examples/pubsub",
        Command::new("cargo")
            .arg("run")
            .arg("--bin")
            .arg("subscriber"),
    );

    rosrust::init("hello_world_talker_listener");

    let (tx, rx) = unbounded();

    let _log_subscriber =
        rosrust::subscribe::<msg::std_msgs::String, _>("chatter", 100, move |data| {
            tx.send((2, data.data)).unwrap();
        })
        .unwrap();

    let publisher = rosrust::publish::<msg::std_msgs::String>("chatter", 100).unwrap();

    let mut message = msg::std_msgs::String::default();
    message.data = "hello world".to_owned();

    util::test_publisher(&publisher, &message, &rx, r"^hello world", 50);
}
