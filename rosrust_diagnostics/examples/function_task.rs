use rosrust_diagnostics::{FunctionExt, Level, Status, Updater};

fn main() {
    // Initialize ROS node
    rosrust::init("function_task_example");

    // Create updater that automatically connects to the diagnostics topic
    let mut updater = Updater::new().unwrap();

    // Create simple task and add it to the updater
    updater.add_task(
        (|status: &mut Status| {
            status.set_summary(Level::Warn, "foo");
            status.add("one".into(), 1);
            status.add("true_bool".into(), true);
        })
        .into_task("my_task"),
    );

    let mut rate = rosrust::rate(1.0);

    while rosrust::is_ok() {
        // Publish diagnostic update
        updater.update().unwrap();
        rate.sleep();
    }
}
