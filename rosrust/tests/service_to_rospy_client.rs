use std::process::Command;

mod util;

mod msg {
    rosrust::rosmsg_include!(rospy_tutorials / AddTwoInts);
}

fn test_request(a: i64, b: i64) {
    let client = Command::new("rosrun")
        .arg("rospy_tutorials")
        .arg("add_two_ints_client")
        .arg(format!("{}", a))
        .arg(format!("{}", b))
        .output()
        .unwrap();
    util::assert_success_and_output_containing(client, &format!("{} + {} = {}", a, b, a + b));
}

#[test]
fn service_to_rospy_client() {
    let _roscore = util::run_roscore_for(util::TestVariant::ServiceToRospyClient);

    rosrust::init("add_two_ints_service");
    let _service = rosrust::service::<msg::rospy_tutorials::AddTwoInts, _>("add_two_ints", |req| {
        Ok(msg::rospy_tutorials::AddTwoIntsRes { sum: req.a + req.b })
    })
    .unwrap();

    test_request(0, 10);
    test_request(10, 0);
    test_request(100, -200);
}
