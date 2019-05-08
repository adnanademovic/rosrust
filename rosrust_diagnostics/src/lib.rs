/*!
This crate provides handling of diagnostics for `rosrust`.

Functionality is made to be as close as possible to the ROS [diagnostic updater] while trying to
support ownership models with the minimum amounts of allocation and dynamic interfaces.

[diagnostic updater]: http://wiki.ros.org/diagnostic_updater
*/
#![deny(missing_docs)]

pub use composite_task::{CompositeTask, CompositeTaskRunner};
pub use function_task::{FunctionExt, FunctionTask};
pub use msg::diagnostic_msgs::KeyValue;
pub use status::Status;
pub use task::Task;
pub use tasks::{FrequencyStatus, Heartbeat, TimestampStatus};
pub use updater::{Updater, UpdaterLowLevelExt};

mod composite_task;
mod function_task;
#[macro_use]
mod macros;
pub mod msg;
mod status;
mod task;
pub mod tasks;
mod updater;

/// Possible levels of operations in a diagnostic status.
#[repr(i8)]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Level {
    /// The diagnostic concluded that all checks passed.
    Ok = msg::diagnostic_msgs::DiagnosticStatus::OK,
    /// Checks resulted in a warning.
    Warn = msg::diagnostic_msgs::DiagnosticStatus::WARN,
    /// Checks determined an error happened.
    Error = msg::diagnostic_msgs::DiagnosticStatus::ERROR,
}
