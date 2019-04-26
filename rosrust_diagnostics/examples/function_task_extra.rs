use rosrust_diagnostics::{FunctionExt, Level, Status, Updater};

fn main() {
    // Initialize ROS node
    rosrust::init("function_task_example");

    // Create updater that automatically connects to the diagnostics topic
    let mut updater = Updater::new().unwrap();

    // Create simple task and add it to the updater
    let my_task = (|status: &mut Status| {
        status.set_summary(Level::Warn, "foo");
        status.add("one", 1);
        status.add("true_bool", true);
    })
    .into_task("my_task");
    updater.add_task(&my_task).unwrap();

    // Create simple tasks

    let task1 = (|status: &mut Status| {
        status.set_summary(Level::Error, "bar");
        status.add("two", 2);
    })
    .into_task("my_task1");

    let task2 = (|status: &mut Status| {
        status.set_summary(Level::Warn, "baz");
        status.add("three", 3);
    })
    .into_task("my_task2");

    let mut rate = rosrust::rate(1.0);

    while rosrust::is_ok() {
        // Publish diagnostic update
        updater.update_with_extra(&[&task1, &task2]).unwrap();
        rate.sleep();
    }
}
