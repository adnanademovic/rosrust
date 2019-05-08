pub mod lossy_channel;

pub static FAILED_TO_LOCK: &'static str = "Failed to acquire lock";
pub static CROSSBEAM_CHANNEL_UNEXPECTEDLY_CLOSED: &'static str =
    "Crossbeam channel unexpectedly closed on one end";
