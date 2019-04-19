pub use composite_task::CompositeTask;
pub use function_task::{FunctionExt, FunctionTask};
pub use msg::diagnostic_msgs::KeyValue;
pub use status::Status;
pub use task::Task;

mod composite_task;
mod function_task;
pub mod msg;
mod status;
mod task;

#[repr(i8)]
#[derive(Copy, Clone, Debug)]
pub enum Level {
    Ok = msg::diagnostic_msgs::DiagnosticStatus::OK,
    Warn = msg::diagnostic_msgs::DiagnosticStatus::WARN,
    Error = msg::diagnostic_msgs::DiagnosticStatus::ERROR,
    Stale = msg::diagnostic_msgs::DiagnosticStatus::STALE,
}
