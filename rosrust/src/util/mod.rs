pub mod lossy_channel;

pub static FAILED_TO_LOCK: &'static str = "Failed to acquire lock";
pub static MPSC_CHANNEL_UNEXPECTEDLY_CLOSED: &'static str =
    "MPSC channel unexpectedly closed on one end";
