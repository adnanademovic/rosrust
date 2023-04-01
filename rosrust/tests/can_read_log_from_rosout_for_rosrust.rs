use crossbeam::channel::unbounded;
use std::process::Command;

mod util;

mod msg {
    rosrust::rosmsg_include!(rosgraph_msgs / Log);
}

#[test]
fn can_read_log_from_rosout_for_rosrust() {
    let _roscore = util::run_roscore_for(util::TestVariant::CanReadLogFromRosoutForRosrust);
    let _publisher = util::ChildProcessTerminator::spawn_example(
        Command::new("cargo")
            .arg("run")
            .arg("--example")
            .arg("publisher"),
    );

    rosrust::init("rosout_agg_listener");

    let (tx, rx) = unbounded();

    let _subscriber =
        rosrust::subscribe::<msg::rosgraph_msgs::Log, _>("/rosout_agg", 100, move |data| {
            tx.send((data.level, data.msg)).unwrap();
        })
        .unwrap();

    let rate = rosrust::rate(1.0);

    for _ in 0..10 {
        for (level, message) in rx.try_iter() {
            println!("Received message at level {}: {}", level, message);
            if level == 2 && message.contains("hello world") {
                return;
            }
        }
        rate.sleep();
    }

    panic!("Failed to receive data on /rosout_agg");
}
