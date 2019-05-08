use crate::{Status, Task};

/// A diagnostic task based on a function.
///
/// The task calls the function when it updates. The function should update the passed in status
/// and collect data.
///
/// This is useful for gathering information about a device or driver, like temperature,
/// calibration, etc.
///
/// This is the simplest method of creating tasks.
///
/// You can call `.into_task(name)` from `FunctionExt` on any function with the correct signature
/// to create a function task easily.
#[derive(Clone)]
pub struct FunctionTask<F>
where
    F: Fn(&mut Status),
{
    name: String,
    function: F,
}

impl<F> FunctionTask<F>
where
    F: Fn(&mut Status),
{
    /// Create a function task with the given name and function.
    #[inline]
    pub fn new(name: &str, function: F) -> Self {
        Self {
            name: name.into(),
            function,
        }
    }
}

impl<F> Task for FunctionTask<F>
where
    F: Fn(&mut Status),
{
    #[inline]
    fn name(&self) -> &str {
        &self.name
    }

    #[inline]
    fn run(&self, status: &mut Status) {
        (self.function)(status)
    }
}

/// Extension trait for functions that allows easy creation of function tasks.
pub trait FunctionExt: Sized + Fn(&mut Status) {
    /// Converts the function into a task, and gives it the provided name.
    fn into_task(self, name: &str) -> FunctionTask<Self>;
}

impl<F> FunctionExt for F
where
    F: Fn(&mut Status),
{
    fn into_task(self, name: &str) -> FunctionTask<Self> {
        FunctionTask::new(name, self)
    }
}
