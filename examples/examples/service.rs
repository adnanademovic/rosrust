use env_logger;
use rosrust;

fn main() {
    env_logger::init();

    // Initialize node
    rosrust::init("add_two_ints_server");

    // Create service
    // The service is stopped when the returned object is destroyed
    let _service_raii =
        rosrust::service::<rosrust_msg::roscpp_tutorials::TwoInts, _>("add_two_ints", move |req| {
            // Callback for handling requests
            let sum = req.a + req.b;

            // Log each request
            rosrust::ros_info!("{} + {} = {}", req.a, req.b, sum);

            Ok(rosrust_msg::roscpp_tutorials::TwoIntsRes { sum })
        })
        .unwrap();

    // Block the thread until a shutdown signal is received
    rosrust::spin();
}
