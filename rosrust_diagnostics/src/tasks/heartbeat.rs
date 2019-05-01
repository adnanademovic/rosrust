use crate::{Level, Status, Task};

pub struct Heartbeat;

impl Task for Heartbeat {
    fn name(&self) -> &str {
        "Heartbeat"
    }

    fn run(&self, status: &mut Status) {
        status.set_summary(Level::Ok, "Alive");
    }
}
