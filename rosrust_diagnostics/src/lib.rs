pub use composite_task::{CompositeTask, CompositeTaskRunner};
pub use function_task::{FunctionExt, FunctionTask};
pub use msg::diagnostic_msgs::KeyValue;
pub use status::Status;
pub use task::Task;
pub use tasks::{FrequencyStatus, Heartbeat};
pub use updater::Updater;

mod composite_task;
mod function_task;
#[macro_use]
mod macros;
pub mod msg;
mod status;
mod task;
pub mod tasks;
mod updater;

#[repr(i8)]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Level {
    Ok = msg::diagnostic_msgs::DiagnosticStatus::OK,
    Warn = msg::diagnostic_msgs::DiagnosticStatus::WARN,
    Error = msg::diagnostic_msgs::DiagnosticStatus::ERROR,
    Stale = msg::diagnostic_msgs::DiagnosticStatus::STALE,
}
