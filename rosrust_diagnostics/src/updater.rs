use crate::msg::diagnostic_msgs::{DiagnosticArray, DiagnosticStatus};
use crate::msg::std_msgs::Header;
use crate::{Level, Status, Task};
use rosrust::{error::Result, Publisher};

pub struct Updater {
    publisher: Publisher<DiagnosticArray>,
    tasks: Vec<Box<dyn Task>>,
    hardware_id: String,
}

impl Updater {
    pub fn new() -> Result<Self> {
        let publisher = rosrust::publish("/diagnostics", 10)?;
        Ok(Self {
            publisher,
            tasks: vec![],
            hardware_id: String::new(),
        })
    }

    pub fn add_task(&mut self, task: impl Task + 'static) {
        self.tasks.push(Box::new(task))
    }

    pub fn update_with_extra<'a>(&'a self, extra_tasks: &[&'a dyn Task]) -> Result<()> {
        self.publish(self.perform_checks(extra_tasks))
    }

    pub fn update(&self) -> Result<()> {
        self.update_with_extra(&vec![])
    }

    pub fn perform_checks<'a>(&self, extra_tasks: &[&'a dyn Task]) -> Vec<DiagnosticStatus> {
        Iterator::chain(
            self.tasks.iter().map(|v| &(**v)),
            extra_tasks.into_iter().map(|v| *v),
        )
        .map(|task| self.perform_passed_check(task))
        .collect()
    }

    pub fn perform_passed_check(&self, task: &dyn Task) -> DiagnosticStatus {
        let mut status = Status {
            name: task.name().into(),
            hardware_id: self.hardware_id.clone(),
            level: Level::Error,
            message: "No message was set".into(),
            values: vec![],
        };
        task.run(&mut status);
        status.into()
    }

    pub fn publish(&self, status: Vec<DiagnosticStatus>) -> Result<()> {
        let message = DiagnosticArray {
            header: Header::default(),
            status,
        };
        self.publisher.send(message)
    }
}
