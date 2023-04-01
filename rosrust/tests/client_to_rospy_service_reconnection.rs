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
fn client_to_rospy_service_reconnection() {
    let _roscore = util::run_roscore_for(util::TestVariant::ClientToRospyServiceReconnection);
    rosrust::init("add_two_ints_client");
    let original_service = util::ChildProcessTerminator::spawn(
        Command::new("rosrun")
            .arg("rospy_tutorials")
            .arg("add_two_ints_server"),
    );

    let client = rosrust::client::<msg::rospy_tutorials::AddTwoInts>("add_two_ints").unwrap();
    test_request(&client, 0, 10);
    test_request(&client, 10, 0);
    test_request(&client, 100, -200);
    mem::drop(original_service);
    rosrust::sleep(Duration::from_seconds(1));

    let request_that_should_fail = client.req(&msg::rospy_tutorials::AddTwoIntsReq { a: 5, b: 10 });
    assert!(
        request_that_should_fail.is_err(),
        "Request should time out at some point and fail if the service does not exist"
    );

    let request_that_should_succeed =
        client.req_async(msg::rospy_tutorials::AddTwoIntsReq { a: 5, b: 10 });
    let _replacement_service = util::ChildProcessTerminator::spawn(
        Command::new("rosrun")
            .arg("rospy_tutorials")
            .arg("add_two_ints_server"),
    );
    assert_eq!(
        15,
        request_that_should_succeed
            .read()
            .expect("Request should have found the new service in the repeated tries")
            .unwrap()
            .sum,
    );
}
