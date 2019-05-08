use rosrust::Duration;
use rosrust_diagnostics::{FrequencyStatus, Updater};
use std::sync::Arc;

fn main() {
    // Initialize ROS node
    rosrust::init("frequency_status_example");

    // Create updater that automatically connects to the diagnostics topic
    let mut updater = Updater::new().unwrap();
    updater.set_verbose(true);

    let mut freq_statuses = vec![];

    freq_statuses.push(Arc::new(
        FrequencyStatus::builder().name("No limits").build(),
    ));
    freq_statuses.push(Arc::new(
        FrequencyStatus::builder()
            .name("Only max")
            .max_frequency(10.0)
            .build(),
    ));
    freq_statuses.push(Arc::new(
        FrequencyStatus::builder()
            .name("Only min")
            .min_frequency(5.0)
            .build(),
    ));
    freq_statuses.push(Arc::new(
        FrequencyStatus::builder()
            .name("Both limits")
            .min_frequency(5.0)
            .max_frequency(10.0)
            .build(),
    ));

    let updater_freq_statuses = freq_statuses.clone();
    for task in &updater_freq_statuses {
        updater.add_task(&**task).unwrap();
    }

    let delay_param = rosrust::param("~delay").unwrap();

    let tick_thread = std::thread::spawn(move || {
        while rosrust::is_ok() {
            // Wait for time passed as a delay, or wait one second
            let delay_seconds = delay_param.get().unwrap_or(1.0);
            rosrust::sleep(Duration::from_nanos(
                (delay_seconds * 1_000_000_000.0) as i64,
            ));
            for task in &freq_statuses {
                task.tick();
            }
        }
    });

    let mut rate = rosrust::rate(100.0);

    while rosrust::is_ok() {
        updater.update().unwrap();
        rate.sleep();
    }

    tick_thread.join().unwrap();
}
