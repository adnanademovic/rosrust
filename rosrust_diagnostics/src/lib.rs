pub use msg::diagnostic_msgs::KeyValue;
pub use status::Status;

mod msg;
mod status;

#[repr(i8)]
#[derive(Copy, Clone, Debug)]
pub enum Level {
    Ok = msg::diagnostic_msgs::DiagnosticStatus::OK,
    Warn = msg::diagnostic_msgs::DiagnosticStatus::WARN,
    Error = msg::diagnostic_msgs::DiagnosticStatus::ERROR,
    Stale = msg::diagnostic_msgs::DiagnosticStatus::STALE,
}
