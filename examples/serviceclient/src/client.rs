extern crate env_logger;
#[macro_use]
extern crate rosrust;

use rosrust::Ros;
use std::{env, thread, time};

rosmsg_include!();

fn main() {
    env_logger::init().unwrap();

    let args: Vec<_> = env::args().collect();

    if args.len() != 3 {
        println!("usage: client X Y");
        return;
    }

    let a = args[1].parse::<i64>().unwrap();
    let b = args[2].parse::<i64>().unwrap();

    let ros = Ros::new("add_two_ints_client").unwrap();
    let client = ros.client::<msg::roscpp_tutorials::TwoInts>("add_two_ints")
        .unwrap();

    // Sync approach
    println!(
        "{} + {} = {}",
        a,
        b,
        client
            .req(&msg::roscpp_tutorials::TwoIntsReq { a, b })
            .unwrap()
            .unwrap()
            .sum
    );

    // Async approach
    let retval = client.req_async(msg::roscpp_tutorials::TwoIntsReq { a, b });
    println!("{} + {} = {}", a, b, retval.read().unwrap().unwrap().sum);

    thread::sleep(time::Duration::from_secs(1));
}
