use crate::{Level, Status, Task};

/// Diagnostic task to monitor whether a node is alive.
///
/// This diagnostic task always reports as OK and "Alive" when it runs.
pub struct Heartbeat;

impl Task for Heartbeat {
    fn name(&self) -> &str {
        "Heartbeat"
    }

    fn run(&self, status: &mut Status) {
        status.set_summary(Level::Ok, "Alive");
    }
}
