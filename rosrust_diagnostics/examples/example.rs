//! Example equivalent to the the example for `roscpp` and `rospy`.
//!
//! Example this is based on: http://docs.ros.org/api/diagnostic_updater/html/example_8py_source.html

use rosrust_diagnostics::{CompositeTask, FunctionExt, Level, Status, Task, Updater};
use std::sync::atomic::{AtomicUsize, Ordering};

static TIME_TO_LAUNCH: AtomicUsize = AtomicUsize::new(0);

fn dummy_diagnostic(status: &mut Status) {
    let time_to_launch = TIME_TO_LAUNCH.load(Ordering::SeqCst);
    if time_to_launch < 10 {
        status.set_summary(
            Level::Error,
            format!(
                "Buckle your seat belt. Launch in {} seconds!",
                time_to_launch
            ),
        );
    } else {
        status.set_summary(Level::Ok, "Launch is in a long time. Have a soda.");
    }

    status.add("Diagnostic Name", "dummy");
    status.add("Time to Launch", time_to_launch);
    status.add(
        "Geeky thing to say",
        format!(
            "The square of the time to launch {} is {}",
            time_to_launch,
            time_to_launch * time_to_launch
        ),
    );
}

struct DummyTask;

impl Task for DummyTask {
    fn name(&self) -> &str {
        "Updater Derived from Task"
    }

    fn run(&self, status: &mut Status) {
        status.set_summary(Level::Warn, "This is a silly updater.");
        status.add("Stupidicity of this updater", 1000.0);
    }
}

fn check_lower_bound(status: &mut Status) {
    let time_to_launch = TIME_TO_LAUNCH.load(Ordering::SeqCst);
    if time_to_launch > 5 {
        status.set_summary(Level::Ok, "Lower-bound OK");
        status.add("Low-Side Margin", time_to_launch - 5);
    } else {
        status.set_summary(Level::Error, "Too low");
        status.add("Low-Side Margin", 0);
    }
}

fn check_upper_bound(status: &mut Status) {
    let time_to_launch = TIME_TO_LAUNCH.load(Ordering::SeqCst);
    if time_to_launch < 10 {
        status.set_summary(Level::Ok, "Upper-bound OK");
        status.add("Top-Side Margin", 10 - time_to_launch);
    } else {
        status.set_summary(Level::Warn, "Too high");
        status.add("Top-Side Margin", 0);
    }
}

fn main() {
    rosrust::init("rosrust_diagnostics_example");

    let mut updater = Updater::new().unwrap();
    updater.set_hardware_id("none");
    updater.set_verbose(true);

    let function_updater = dummy_diagnostic.into_task("Function updater");
    updater.add_task(&function_updater).unwrap();

    let dummy_task = DummyTask;
    let dummy_task2 = DummyTask;

    updater.add_task(&dummy_task).unwrap();

    let mut bounds = CompositeTask::new("Bound check");
    bounds.add_task(check_lower_bound.into_task("Lower-bound check"));
    bounds.add_task(check_upper_bound.into_task("Upper-bound check"));

    updater.add_task(&bounds).unwrap();

    let rate = rosrust::rate(10.0);

    while rosrust::is_ok() {
        updater.force_update_with_extra(&[&dummy_task2]).unwrap();
        rate.sleep();
    }
}
