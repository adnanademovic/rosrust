pub use self::clock::{Clock, Delay, Rate};
pub use self::master::{SystemState, Topic};
pub use self::ros::{Parameter, Ros};
use std::sync::atomic::{AtomicBool, Ordering};

mod clock;
pub mod error;
mod master;
mod naming;
pub mod raii;
pub mod resolve;
mod ros;
mod slave;

pub struct ShutdownManager {
    should_shutdown: AtomicBool,
}

impl Default for ShutdownManager {
    fn default() -> Self {
        Self {
            should_shutdown: AtomicBool::new(false),
        }
    }
}

impl ShutdownManager {
    pub fn awaiting_shutdown(&self) -> bool {
        self.should_shutdown.load(Ordering::Relaxed)
    }

    pub fn shutdown(&self) {
        self.should_shutdown.store(true, Ordering::Relaxed)
    }
}
