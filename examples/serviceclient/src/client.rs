extern crate env_logger;
#[macro_use]
extern crate rosrust;
#[macro_use]
extern crate rosrust_codegen;

use std::time;

rosmsg_include!();

fn main() {
    env_logger::init();

    // Fetch args that are not meant for rosrust
    let args: Vec<_> = rosrust::args();

    if args.len() != 3 {
        eprintln!("usage: client X Y");
        return;
    }

    let a = args[1].parse::<i64>().unwrap();
    let b = args[2].parse::<i64>().unwrap();

    // Initialize node
    rosrust::init("add_two_ints_client");

    // Wait ten seconds for the service to appear
    rosrust::wait_for_service("add_two_ints", Some(time::Duration::from_secs(10))).unwrap();

    // Create client for the service
    let client = rosrust::client::<msg::roscpp_tutorials::TwoInts>("add_two_ints").unwrap();

    // Synchronous call that blocks the thread until a response is received
    ros_info!(
        "{} + {} = {}",
        a,
        b,
        client
            .req(&msg::roscpp_tutorials::TwoIntsReq { a, b })
            .unwrap()
            .unwrap()
            .sum
    );

    // Asynchronous call that can be resolved later on
    let retval = client.req_async(msg::roscpp_tutorials::TwoIntsReq { a, b });
    ros_info!("{} + {} = {}", a, b, retval.read().unwrap().unwrap().sum);
}
