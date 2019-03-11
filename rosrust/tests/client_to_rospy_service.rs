use rosrust;
use std::process::Command;
use std::time;

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
fn client_to_rospy_service() {
    let _roscore = util::run_roscore(11313);
    let _service = util::ChildProcessTerminator::spawn(
        Command::new("rosrun")
            .arg("rospy_tutorials")
            .arg("add_two_ints_server"),
    );

    rosrust::init("add_two_ints_client");
    rosrust::wait_for_service("add_two_ints", Some(time::Duration::from_secs(10))).unwrap();
    let client = rosrust::client::<msg::rospy_tutorials::AddTwoInts>("add_two_ints").unwrap();

    test_request(&client, 0, 10);
    test_request(&client, 10, 0);
    test_request(&client, 100, -200);
}
