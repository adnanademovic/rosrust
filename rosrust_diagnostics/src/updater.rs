use crate::msg::diagnostic_msgs::{DiagnosticArray, DiagnosticStatus};
use crate::msg::std_msgs::Header;
use crate::{Level, Status, Task};
use rosrust::{error::Result, Publisher};

pub struct Updater {
    publisher: Publisher<DiagnosticArray>,
    tasks: Vec<Box<dyn Task>>,
    hardware_id: String,
    verbose: bool,
}

impl Updater {
    pub fn new() -> Result<Self> {
        let publisher = rosrust::publish("/diagnostics", 10)?;
        Ok(Self {
            publisher,
            tasks: vec![],
            hardware_id: "none".into(),
            verbose: false,
        })
    }

    #[inline]
    pub fn set_hardware_id(&mut self, hardware_id: impl std::string::ToString) {
        self.hardware_id = hardware_id.to_string();
    }

    #[inline]
    pub fn get_hardware_id(&self) -> &str {
        &self.hardware_id
    }

    #[inline]
    pub fn set_verbose(&mut self, verbose: bool) {
        self.verbose = verbose;
    }

    #[inline]
    pub fn is_verbose(&self) -> bool {
        self.verbose
    }

    #[inline]
    pub fn add_task(&mut self, task: impl Task + 'static) -> Result<()> {
        let advertisement_result = self.advertise_added_task(&task);
        self.tasks.push(Box::new(task));
        advertisement_result
    }

    pub fn advertise_added_task(&mut self, task: &dyn Task) -> Result<()> {
        let status = self.make_broadcast_status_for(task, Level::Ok, "Node starting up");
        self.publish(vec![status])
    }

    pub fn remove_task(&mut self, name: &str) {
        self.tasks.retain(|task| task.name() != name);
    }

    #[inline]
    pub fn update(&self) -> Result<()> {
        self.update_with_extra(&[])
    }

    #[inline]
    pub fn update_with_extra<'a>(&'a self, extra_tasks: &[&'a dyn Task]) -> Result<()> {
        self.publish(self.make_update_statuses(extra_tasks))
    }

    #[inline]
    pub fn make_update_statuses<'a>(&self, extra_tasks: &[&'a dyn Task]) -> Vec<DiagnosticStatus> {
        self.map_over_tasks(extra_tasks, |task| self.make_update_status(task))
    }

    pub fn make_update_status(&self, task: &dyn Task) -> DiagnosticStatus {
        let mut status = Status {
            name: task.name().into(),
            hardware_id: self.hardware_id.clone(),
            level: Level::Error,
            message: "No message was set".into(),
            values: vec![],
        };
        task.run(&mut status);
        if self.verbose && status.level != Level::Ok {
            rosrust::ros_warn!(
                "Non-zero diagnostic status. Name: '{}', status {}: '{}'",
                status.name,
                status.level as i8,
                status.message,
            );
        }
        status.into()
    }

    #[inline]
    pub fn broadcast(&self, level: Level, message: &str) -> Result<()> {
        self.broadcast_with_extra(&[], level, message)
    }

    #[inline]
    pub fn broadcast_with_extra<'a>(
        &'a self,
        extra_tasks: &[&'a dyn Task],
        level: Level,
        message: &str,
    ) -> Result<()> {
        self.publish(self.make_broadcast_statuses(extra_tasks, level, message))
    }

    #[inline]
    pub fn make_broadcast_statuses<'a>(
        &self,
        extra_tasks: &[&'a dyn Task],
        level: Level,
        message: &str,
    ) -> Vec<DiagnosticStatus> {
        self.map_over_tasks(extra_tasks, |task| {
            self.make_broadcast_status_for(task, level, message)
        })
    }

    #[inline]
    pub fn make_broadcast_status_for(
        &self,
        task: &dyn Task,
        level: Level,
        message: &str,
    ) -> DiagnosticStatus {
        Status {
            name: task.name().into(),
            hardware_id: self.hardware_id.clone(),
            level,
            message: message.into(),
            values: vec![],
        }
        .into()
    }

    #[inline]
    pub fn publish(&self, status: Vec<DiagnosticStatus>) -> Result<()> {
        let message = DiagnosticArray {
            header: Header::default(),
            status,
        };
        self.publisher.send(message)
    }

    fn map_over_tasks<'a, F>(
        &self,
        extra_tasks: &[&'a dyn Task],
        handler: F,
    ) -> Vec<DiagnosticStatus>
    where
        F: Fn(&dyn Task) -> DiagnosticStatus,
    {
        Iterator::chain(
            self.tasks.iter().map(|v| &(**v)),
            extra_tasks.iter().cloned(),
        )
        .map(handler)
        .collect()
    }
}
