use crate::{Status, Task};

pub struct CompositeTask<'a> {
    name: &'a str,
    tasks: Vec<&'a Task>,
}

impl<'a> CompositeTask<'a> {
    pub fn new(name: &'a str) -> Self {
        Self {
            name,
            tasks: vec![],
        }
    }

    pub fn add_task(&mut self, task: &'a Task) {
        self.tasks.push(task)
    }
}

impl<'a> Task for CompositeTask<'a> {
    #[inline]
    fn name(&self) -> &str {
        self.name
    }

    fn run(&self, status: &mut Status) {
        let mut combined_summary = Status::default();
        let original_level = status.level;
        let original_message = status.message.clone();

        for task in &self.tasks {
            status.set_summary(original_level, &original_message);
            task.run(status);
            combined_summary.merge_summary_with(status);
        }
        status.copy_summary(&combined_summary);
    }
}
