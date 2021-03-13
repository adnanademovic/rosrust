use rosrust_actionlib::SimpleActionClient;
use rosrust_msg::actionlib_tutorials as alt;
use std::sync::mpsc::channel;

fn fibonacci_client() {
    let mut client = SimpleActionClient::<alt::FibonacciAction>::new("fibonacci").unwrap();
    client.wait_for_server(None);
    let (tx, rx) = channel();
    let mut goal_handles = vec![];
    for order in &[5, 10, 20] {
        let order = *order;
        let goal = alt::FibonacciGoal { order };
        let tx = tx.clone();
        client
            .build_goal_sender(goal)
            .on_done(move |state, result| {
                rosrust::ros_info!(
                    "Goal {} Done with status {:?} and result: {:?}",
                    order,
                    state,
                    result
                );
                tx.send(()).unwrap();
            })
            .on_active(move || {
                rosrust::ros_info!("Goal {} became active", order);
            })
            .on_feedback(move |feedback| {
                rosrust::ros_info!("Received feedback for {}: {:?}", order, feedback);
            })
            .send();
        goal_handles.push(client.detach_goal());
    }
    rx.recv().unwrap();
    rx.recv().unwrap();
    rx.recv().unwrap();
}

fn main() {
    rosrust::init("fibonacci_client");

    fibonacci_client();
}
