/// Runs diagnostics over a set of tasks, the way a composite task would.
///
/// This macro allows defining custom `Task` implementers as composite tasks easily,
/// without requiring any heap allocations or moves.
///
/// # Examples
///
/// A task consisting of a heartbeat and an arbitrary task.
///
/// ```
/// use rosrust_diagnostics::{Heartbeat, Task, Status, Level, run_diagnostics, FunctionExt};
///
/// struct ExampleTask<T: Task> {
///     heartbeat: Heartbeat,
///     other: T,
/// }
///
/// impl<T: Task> Task for ExampleTask<T> {
///     fn name(&self) -> &str {
///         "Example task"
///     }
///
///     fn run(&self, status: &mut Status) {
///         run_diagnostics!(status, self.heartbeat, self.other);
///     }
/// }
///
/// fn example_fn(status: &mut Status) {
///     status.set_summary(Level::Ok, "foobar");
/// }
///
/// let task = ExampleTask {
///     heartbeat: Heartbeat,
///     other: example_fn.into_task("fn_task"),
/// };
///
/// let mut status = Status::default();
/// task.run(&mut status);
///
/// assert_eq!(status.level, Level::Ok);
/// assert_eq!(status.message, "Alive; foobar");
/// ```
#[macro_export]
macro_rules! run_diagnostics {
    ($status: expr, $($tasks:expr),*) => {
        {
            let mut runner = $crate::CompositeTaskRunner::new($status);
            $(
                runner.run(&$tasks);
            )*
            runner.finish();
        }
    };
}
