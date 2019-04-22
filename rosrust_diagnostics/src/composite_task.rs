use crate::{Level, Status, Task};

pub struct CompositeTask<'a> {
    name: &'a str,
    tasks: Vec<&'a dyn Task>,
}

impl<'a> CompositeTask<'a> {
    pub fn new(name: &'a str) -> Self {
        Self {
            name,
            tasks: vec![],
        }
    }

    pub fn add_task(&mut self, task: &'a dyn Task) {
        self.tasks.push(task)
    }
}

impl<'a> Task for CompositeTask<'a> {
    #[inline]
    fn name(&self) -> &str {
        self.name
    }

    fn run(&self, status: &mut Status) {
        let mut runner = CompositeTaskRunner::new(status);

        for task in &self.tasks {
            runner.run(*task);
        }
    }
}

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
    pub fn new(target: &'a mut Status) -> Self {
        Self {
            level: target.level,
            message: target.message.clone(),
            target,
            combination: Status::default(),
            finished: false,
        }
    }

    pub fn run(&mut self, task: &dyn Task) {
        self.target.set_summary(self.level, &self.message);
        task.run(&mut self.target);
        self.combination.merge_summary_with(&self.target);
    }

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
