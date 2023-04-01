use std::process::Command;

mod util;

mod msg {
    rosrust::rosmsg_include!(roscpp_tutorials / TwoInts);
}

fn test_request(a: i64, b: i64) {
    let client = Command::new("rosrun")
        .arg("roscpp_tutorials")
        .arg("add_two_ints_client")
        .arg(format!("{}", a))
        .arg(format!("{}", b))
        .output()
        .unwrap();
    util::assert_success_and_output_containing(client, &format!("Sum: {}", a + b));
}

#[test]
fn service_to_roscpp_client() {
    let _roscore = util::run_roscore_for(util::TestVariant::ServiceToRoscppClient);

    rosrust::init("add_two_ints_service");
    let _service = rosrust::service::<msg::roscpp_tutorials::TwoInts, _>("add_two_ints", |req| {
        Ok(msg::roscpp_tutorials::TwoIntsRes { sum: req.a + req.b })
    })
    .unwrap();

    test_request(0, 10);
    test_request(10, 0);
    test_request(100, -200);
}
