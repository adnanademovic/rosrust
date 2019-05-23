use crate::GoalID;
use rosrust;
use std::sync::atomic::{AtomicUsize, Ordering};

pub struct GoalIdGenerator {
    name: String,
}

impl Default for GoalIdGenerator {
    #[inline]
    fn default() -> Self {
        Self {
            name: rosrust::name(),
        }
    }
}

static GOAL_COUNT: AtomicUsize = AtomicUsize::new(1);

impl GoalIdGenerator {
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }

    #[inline]
    pub fn generate_id(&self) -> GoalID {
        let seq_id = GOAL_COUNT.fetch_add(1, Ordering::SeqCst);
        let stamp = rosrust::now();
        let id = format!(
            "{name}-{seq_id}-{secs}.{nsecs}",
            name = self.name,
            seq_id = seq_id,
            secs = stamp.sec,
            nsecs = stamp.nsec,
        );

        GoalID { id, stamp }
    }
}
