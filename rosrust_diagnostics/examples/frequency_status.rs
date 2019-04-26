use rosrust::Duration;
use rosrust_diagnostics::{FrequencyStatus, Updater};

fn main() {
    // Initialize ROS node
    rosrust::init("frequency_status_example");

    // Create updater that automatically connects to the diagnostics topic
    let mut updater = Updater::new().unwrap();
    updater.set_verbose(true);

    let ticker = FrequencyStatus::create_ticker();

    updater
        .add_task(
            FrequencyStatus::builder()
                .name("No limits")
                .ticker(&ticker)
                .build(),
        )
        .unwrap();
    updater
        .add_task(
            FrequencyStatus::builder()
                .name("Only max")
                .ticker(&ticker)
                .max_frequency(10.0)
                .build(),
        )
        .unwrap();
    updater
        .add_task(
            FrequencyStatus::builder()
                .name("Only min")
                .ticker(&ticker)
                .min_frequency(5.0)
                .build(),
        )
        .unwrap();
    updater
        .add_task(
            FrequencyStatus::builder()
                .name("Both limits")
                .ticker(&ticker)
                .min_frequency(5.0)
                .max_frequency(10.0)
                .build(),
        )
        .unwrap();

    let delay_param = rosrust::param("~delay").unwrap();

    let tick_thread = std::thread::spawn(move || {
        while rosrust::is_ok() {
            // Wait for time passed as a delay, or wait one second
            let delay_seconds = delay_param.get().unwrap_or(1.0);
            rosrust::sleep(Duration::from_nanos(
                (delay_seconds * 1_000_000_000.0) as i64,
            ));
            ticker.tick();
        }
    });

    let mut rate = rosrust::rate(2.0);

    while rosrust::is_ok() {
        updater.update().unwrap();
        rate.sleep();
    }

    tick_thread.join().unwrap();
}
