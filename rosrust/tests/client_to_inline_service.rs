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
fn client_to_inline_service() {
    let _roscore = util::run_roscore_for(util::TestVariant::ClientToInlineService);
    let _service = util::ChildProcessTerminator::spawn_example(
        Command::new("cargo")
            .arg("run")
            .arg("--example")
            .arg("service"),
    );

    rosrust::init("add_two_ints_client_service");

    let _service = rosrust::service::<msg::roscpp_tutorials::TwoInts, _>("add_two_ints", |req| {
        Ok(msg::roscpp_tutorials::TwoIntsRes { sum: req.a + req.b })
    })
    .unwrap();

    let client = rosrust::client::<msg::roscpp_tutorials::TwoInts>("add_two_ints").unwrap();

    test_request(&client, 0, 10);
    test_request(&client, 10, 0);
    test_request(&client, 100, -200);
}
