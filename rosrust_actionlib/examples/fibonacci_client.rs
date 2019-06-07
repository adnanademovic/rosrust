use rosrust_actionlib::SimpleActionClient;

mod msg {
    rosrust::rosmsg_include!(actionlib_tutorials / FibonacciAction);

    rosrust_actionlib::action!(self; actionlib_tutorials: Fibonacci);
}

use msg::actionlib_tutorials as alt;

fn fibonacci_client() -> alt::FibonacciResult {
    let mut client = SimpleActionClient::<alt::FibonacciAction>::new("fibonacci").unwrap();
    client.wait_for_server(None);
    let goal = alt::FibonacciGoal { order: 20 };
    client.build_goal_sender(goal).send();
    client.wait_for_result(None);
    client.result().unwrap()
}

fn main() {
    rosrust::init("fibonacci_client");

    let result = fibonacci_client();

    println!("Result: {:?}", result.sequence);
}
