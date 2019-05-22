use rosrust_actionlib::{ActionServer, ServerSimpleGoalHandle};

mod msg {
    rosrust::rosmsg_include!(actionlib_tutorials / FibonacciAction);

    rosrust_actionlib::action!(self; actionlib_tutorials: Fibonacci);
}

use msg::actionlib_tutorials as alt;

fn handler(gh: ServerSimpleGoalHandle<alt::FibonacciAction>) {
    let rate = rosrust::rate(1.0);

    let mut val1 = 0;
    let mut val2 = 1;
    let mut sequence = vec![val1, val2];

    for _ in 1..gh.goal().order {
        if !rosrust::is_ok() || gh.canceled() {
            rosrust::ros_info!("Action Canceled");
            return;
        }
        let sum = val1 + val2;
        val1 = val2;
        val2 = sum;
        sequence.push(sum);
        gh.handle().publish_feedback(alt::FibonacciFeedback {
            sequence: sequence.clone(),
        });
        rate.sleep();
    }

    rosrust::ros_info!("Action Succeeded");
    gh.build_message()
        .result(alt::FibonacciResult { sequence })
        .send_succeeded();
}

fn main() {
    rosrust::init("fibonacci");

    let _server =
        ActionServer::<alt::FibonacciAction>::new_simple(&rosrust::name(), handler).unwrap();

    rosrust::spin();
}
