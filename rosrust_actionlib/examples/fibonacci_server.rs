use rosrust_actionlib::action_server::ServerSimpleGoalHandle;
use rosrust_actionlib::ActionServer;
use rosrust_msg::actionlib_tutorials as alt;

fn handler(gh: ServerSimpleGoalHandle<alt::FibonacciAction>) {
    let rate = rosrust::rate(1.0);

    let mut val1 = 0;
    let mut val2 = 1;
    let mut sequence = vec![val1, val2];

    for _ in 1..gh.goal().order {
        if !rosrust::is_ok() || gh.canceled() {
            rosrust::ros_info!("Action Canceled");
            gh.response().send_canceled();
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
    gh.response()
        .result(alt::FibonacciResult { sequence })
        .send_succeeded();
}

fn main() {
    rosrust::init("fibonacci");

    let _server =
        ActionServer::<alt::FibonacciAction>::new_simple(&rosrust::name(), handler).unwrap();

    rosrust::spin();
}
