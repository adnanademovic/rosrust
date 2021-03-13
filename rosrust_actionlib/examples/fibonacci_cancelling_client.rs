use rosrust_actionlib::SimpleActionClient;
use rosrust_msg::actionlib_tutorials as alt;

fn fibonacci_client() -> Option<alt::FibonacciResult> {
    let should_cancel = rosrust::param("~cancel").unwrap().get().unwrap_or(false);
    rosrust::ros_info!("Client will cancel? {:?}", should_cancel);
    let mut client = SimpleActionClient::<alt::FibonacciAction>::new("fibonacci").unwrap();
    client.wait_for_server(None);
    let goal = alt::FibonacciGoal { order: 20 };
    client
        .build_goal_sender(goal)
        .on_done(|state, result| {
            rosrust::ros_info!("Done with status {:?} and result: {:?}", state, result);
        })
        .on_active(|| {
            rosrust::ros_info!("Goal became active");
        })
        .on_feedback(|feedback| {
            rosrust::ros_info!("Received feedback: {:?}", feedback);
        })
        .send();
    if should_cancel {
        client.wait_for_result(Some(rosrust::Duration::from_seconds(5)));
        client.cancel_goal();
    }
    client.wait_for_result(None);
    client.result()
}

fn main() {
    rosrust::init("fibonacci_client");

    let result = fibonacci_client();

    if let Some(result) = result {
        println!("Result: {:?}", result.sequence);
    } else {
        println!("Result never came.");
    }
}
