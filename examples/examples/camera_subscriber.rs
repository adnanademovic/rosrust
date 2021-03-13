// Example that requires a lot of processing power to handle all the data received.

use env_logger;
use rosrust;

use std::sync::Mutex;
use std::time::Instant;

fn main() {
    env_logger::init();

    // Initialize node
    rosrust::init("listener");

    let now = Mutex::new(Instant::now());

    // Create subscriber
    // The subscriber is stopped when the returned object is destroyed
    let _subscriber_raii = rosrust::subscribe(
        "/usb_cam/image_raw",
        1,
        move |v: rosrust_msg::sensor_msgs::Image| {
            // Callback for handling received messages
            let mut now = now.lock().unwrap();
            let duration = now.elapsed();
            *now = Instant::now();
            rosrust::ros_info!(
                "Took {}ms to receive image with data amount {} at {:?}",
                duration.as_secs() * 1000 + u64::from(duration.subsec_millis()),
                v.data.len(),
                v.header.stamp,
            );
        },
    )
    .unwrap();

    // Block the thread until a shutdown signal is received
    rosrust::spin();
}
