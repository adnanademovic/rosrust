use std::process::Command;

mod util;

mod msg {
    rosrust::rosmsg_include!(roscpp_tutorials / TwoInts);
}

fn test_request(client: &rosrust::Client<msg::roscpp_tutorials::TwoInts>, a: i64, b: i64) {
    let sum = client
        .req(&msg::roscpp_tutorials::TwoIntsReq { a, b })
        .unwrap()
        .unwrap()
        .sum;
    assert_eq!(a + b, sum);
}

#[test]
fn client_to_rosrust_service() {
    let _roscore = util::run_roscore_for(util::TestVariant::ClientToRosrustService);
    let _service = util::ChildProcessTerminator::spawn_example(
        Command::new("cargo")
            .arg("run")
            .arg("--example")
            .arg("service"),
    );

    rosrust::init("add_two_ints_client");
    let client = rosrust::client::<msg::roscpp_tutorials::TwoInts>("add_two_ints").unwrap();

    test_request(&client, 0, 10);
    test_request(&client, 10, 0);
    test_request(&client, 100, -200);
}
