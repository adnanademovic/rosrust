pub use self::clock::{Clock, Delay, Rate};
pub use self::master::{Master, SystemState, Topic};
pub use self::ros::{Parameter, Ros};
use std::sync::atomic::{AtomicBool, Ordering};

mod clock;
pub mod error;
pub mod handlers;
mod master;
mod naming;
pub mod raii;
pub mod resolve;
mod ros;
mod slave;

pub struct ShutdownManager {
    handler: Box<dyn Fn() + Send + Sync>,
    should_shutdown: AtomicBool,
}

impl ShutdownManager {
    pub fn new(handler: impl Fn() + Send + Sync + 'static) -> Self {
        Self {
            handler: Box::new(handler),
            should_shutdown: AtomicBool::new(false),
        }
    }

    pub fn awaiting_shutdown(&self) -> bool {
        self.should_shutdown.load(Ordering::Relaxed)
    }

    pub fn shutdown(&self) {
        (*self.handler)();
        self.should_shutdown.store(true, Ordering::Relaxed)
    }
}
