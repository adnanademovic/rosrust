use crate::{Level, Status, Task};

/// Merges multiple diagnostic tasks into a single diagnostic task.
///
/// This task allows multiple task instances to be combined into a single task that
/// produces a single single `Status`. The output of the combination has the max of
/// the status levels, and a concatenation of the non-zero-level messages.
///
/// For instance, this could be used to combine the calibration and offset data
/// from an IMU driver.
///
/// This is an easy way of combining task, but it performs heap allocations and
/// takes ownership of tasks. To maintain ownership, and manage things optimally
/// implement your own `Task`, by utilizing the `run_diagnostics!` macro for
/// similar functionality.
pub struct CompositeTask<'a> {
    name: String,
    tasks: Vec<&'a dyn Task>,
}

impl<'a> CompositeTask<'a> {
    /// Creates a new composite task with the given name.
    pub fn new(name: impl std::string::ToString) -> Self {
        Self {
            name: name.to_string(),
            tasks: vec![],
        }
    }

    /// Adds a child to the composite task.
    ///
    /// This child will be called every time the composit task is called.
    pub fn add_task(&mut self, task: &'a dyn Task) {
        self.tasks.push(task)
    }
}

impl<'a> Task for CompositeTask<'a> {
    #[inline]
    fn name(&self) -> &str {
        &self.name
    }

    fn run(&self, status: &mut Status) {
        let mut runner = CompositeTaskRunner::new(status);

        for task in &self.tasks {
            runner.run(&(**task));
        }
    }
}

/// Internal component for the composite task and run diagnostics macro.
pub struct CompositeTaskRunner<'a> {
    level: Level,
    message: String,
    target: &'a mut Status,
    combination: Status,
    finished: bool,
}

impl<'a> Drop for CompositeTaskRunner<'a> {
    fn drop(&mut self) {
        self.apply_summary()
    }
}

impl<'a> CompositeTaskRunner<'a> {
    #[allow(missing_docs)]
    pub fn new(target: &'a mut Status) -> Self {
        Self {
            level: target.level,
            message: target.message.clone(),
            target,
            combination: Status::default(),
            finished: false,
        }
    }

    #[allow(missing_docs)]
    pub fn run(&mut self, task: &dyn Task) {
        self.target.set_summary(self.level, self.message.clone());
        task.run(self.target);
        self.combination.merge_summary_with(self.target);
    }

    #[allow(missing_docs)]
    pub fn finish(mut self) {
        self.apply_summary()
    }

    fn apply_summary(&mut self) {
        if self.finished {
            return;
        }
        self.finished = true;
        self.target.copy_summary(&self.combination);
    }
}
