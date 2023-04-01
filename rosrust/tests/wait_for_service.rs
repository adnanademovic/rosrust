use ros_message::Duration;
use std::mem;
use std::process::Command;

mod util;

mod msg {
    rosrust::rosmsg_include!(rospy_tutorials / AddTwoInts);
}

fn test_request(client: &rosrust::Client<msg::rospy_tutorials::AddTwoInts>, a: i64, b: i64) {
    let sum = client
        .req(&msg::rospy_tutorials::AddTwoIntsReq { a, b })
        .unwrap()
        .unwrap()
        .sum;
    assert_eq!(a + b, sum);
}

#[test]
fn wait_for_service() {
    let _roscore = util::run_roscore_for(util::TestVariant::WaitForService);
    rosrust::init("add_two_ints_client");
    assert!(
        rosrust::wait_for_service("add_two_ints", Some(std::time::Duration::from_secs(1))).is_err(),
        "Waiting for first service should fail",
    );
    let original_service = util::ChildProcessTerminator::spawn(
        Command::new("rosrun")
            .arg("rospy_tutorials")
            .arg("add_two_ints_server"),
    );
    assert!(
        rosrust::wait_for_service("add_two_ints", Some(std::time::Duration::from_secs(10))).is_ok(),
        "Waiting for first service should succeed",
    );

    let client1 = rosrust::client::<msg::rospy_tutorials::AddTwoInts>("add_two_ints").unwrap();
    test_request(&client1, 10, 10);
    mem::drop(client1);
    mem::drop(original_service);

    rosrust::sleep(Duration::from_seconds(1));

    assert!(
        rosrust::wait_for_service("add_two_ints", Some(std::time::Duration::from_secs(1))).is_err(),
        "Waiting for second service should fail",
    );
    let _replacement_service = util::ChildProcessTerminator::spawn(
        Command::new("rosrun")
            .arg("rospy_tutorials")
            .arg("add_two_ints_server"),
    );
    assert!(
        rosrust::wait_for_service("add_two_ints", Some(std::time::Duration::from_secs(10))).is_ok(),
        "Waiting for second service should succeed",
    );
    let client2 = rosrust::client::<msg::rospy_tutorials::AddTwoInts>("add_two_ints").unwrap();
    test_request(&client2, 10, 10);
}
