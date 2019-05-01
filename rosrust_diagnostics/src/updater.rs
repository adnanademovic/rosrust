use crate::msg::diagnostic_msgs::{DiagnosticArray, DiagnosticStatus};
use crate::msg::std_msgs::Header;
use crate::{Level, Status, Task};
use rosrust::{error::Result, Publisher};
use std::sync::Mutex;

/// Manages a list of diagnostic tasks, and calls them in a rate-limited manner.
///
/// This class manages a list of diagnostic tasks. Its `update` function
/// should be called frequently. At some predetermined rate, the `update`
/// function will cause all the diagnostic tasks to run, and will collate
/// and publish the resulting diagnostics.
///
/// The publication rate is determined by the `~diagnostic_period` ROS parameter.
///
/// The class also allows an update to be forced when something significant
/// has happened, and allows a single message to be broadcast on all the
/// diagnostics if normal operation of the node is suspended for some
/// reason.
pub struct Updater<'a> {
    publisher: Publisher<DiagnosticArray>,
    tasks: Vec<&'a dyn Task>,
    hardware_id: String,
    verbose: bool,
    check_tracker: Mutex<CheckTracker>,
}

struct CheckTracker {
    last_update: f64,
    last_check_time_seconds: f64,
    diagnostic_period_seconds: f64,
}

impl CheckTracker {
    fn check_diagnostic_period(&mut self) {
        let now = rosrust::now();
        let now_seconds = now.seconds();
        if now_seconds < self.last_check_time_seconds + 0.25 {
            return;
        }
        self.last_check_time_seconds = now_seconds;
        self.diagnostic_period_seconds = get_diagnostic_period().unwrap_or(1.0);
    }
}

impl<'a> Updater<'a> {
    /// Constructs a new updater.
    ///
    /// The call will fail if creating a publisher for diagnostics fails.
    ///
    /// That failure should only happen if `rosrust::init()` was not called already.
    pub fn new() -> Result<Self> {
        let publisher = rosrust::publish("/diagnostics", 10)?;
        Ok(Self {
            publisher,
            tasks: vec![],
            hardware_id: "none".into(),
            verbose: false,
            check_tracker: Mutex::new(CheckTracker {
                last_update: 0.0,
                last_check_time_seconds: 0.0,
                diagnostic_period_seconds: 1.0,
            }),
        })
    }

    /// Sets the hardware ID.
    #[inline]
    pub fn set_hardware_id(&mut self, hardware_id: impl std::string::ToString) {
        self.hardware_id = hardware_id.to_string();
    }

    /// Gets the hardware ID.
    #[inline]
    pub fn get_hardware_id(&self) -> &str {
        &self.hardware_id
    }

    /// Sets the log verbosity.
    ///
    /// Making the log verbose will output any detected warnings or errors to the log.
    #[inline]
    pub fn set_verbose(&mut self, verbose: bool) {
        self.verbose = verbose;
    }

    /// Gets the log verbosity.
    #[inline]
    pub fn is_verbose(&self) -> bool {
        self.verbose
    }

    /// Adds a task to the updater.
    ///
    /// The operation will be advertised to the diagnostics topic.
    ///
    /// This call will only fail if the advertisement fails. Note that the task is still added,
    /// as the advertisement failing isn't catastrophic.
    #[inline]
    pub fn add_task(&mut self, task: &'a dyn Task) -> Result<()> {
        let advertisement_result = self.advertise_added_task(task);
        self.tasks.push(task);
        advertisement_result
    }

    /// Advertise a task being added.
    ///
    /// You only need to call this if you do not want to add a task to the updater, but instead
    /// want to pass the task as an extra every time.
    ///
    /// Call this method on any task that the updater will not own, but will be called as an extra.
    pub fn advertise_added_task(&self, task: &dyn Task) -> Result<()> {
        let status = self.make_broadcast_status_for(task, Level::Ok, "Node starting up");
        self.publish(vec![status])
    }

    /// Remove any task with the given name.
    pub fn remove_task(&mut self, name: &str) {
        self.tasks.retain(|task| task.name() != name);
    }

    /// Causes the diagnostics to update if the inter-update interval has been exceeded.
    #[inline]
    pub fn update(&self) -> Result<()> {
        self.update_with_extra(&[])
    }

    /// Same as update, but with extra tasks provided to be run.
    ///
    /// If you do not want the updater to share ownership of a task, you can provide it
    /// whenever needed this way.
    ///
    /// Make sure you run `advertise_added_task` for any extra task you decide to pass before
    /// doing any updates.
    #[inline]
    pub fn update_with_extra(&self, extra_tasks: &[&dyn Task]) -> Result<()> {
        if !self.should_update() {
            return Ok(());
        }
        self.force_update_with_extra(extra_tasks)
    }

    fn should_update(&self) -> bool {
        let mut tracker = self.check_tracker.lock().expect(FAILED_TO_LOCK);
        tracker.check_diagnostic_period();
        rosrust::now().seconds() > tracker.last_update + tracker.diagnostic_period_seconds
    }

    fn refresh_last_time(&self) {
        let mut check_tracker = self.check_tracker.lock().expect(FAILED_TO_LOCK);
        check_tracker.last_update = rosrust::now().seconds();
    }

    /// Forces the diagnostics to update.
    ///
    /// Useful if the node has undergone a drastic state change that should be published
    /// immediately.
    #[inline]
    pub fn force_update(&self) -> Result<()> {
        self.force_update_with_extra(&[])
    }

    /// Same as force update, but with extra tasks provided to be run.
    ///
    /// If you do not want the updater to share ownership of a task, you can provide it
    /// whenever needed this way.
    ///
    /// Make sure you run `advertise_added_task` for any extra task you decide to pass before
    /// doing any updates.
    #[inline]
    pub fn force_update_with_extra(&self, extra_tasks: &[&dyn Task]) -> Result<()> {
        self.refresh_last_time();
        self.publish(self.make_update_statuses(extra_tasks))
    }

    /// Outputs a message on all the known tasks.
    ///
    /// Useful if something drastic is happening such as shutdown or a self-test.
    #[inline]
    pub fn broadcast(&self, level: Level, message: &str) -> Result<()> {
        self.broadcast_with_extra(&[], level, message)
    }

    /// Same as broadcast, but with extra tasks provided to be run.
    ///
    /// If you do not want the updater to share ownership of a task, you can provide it
    /// whenever needed this way.
    #[inline]
    pub fn broadcast_with_extra(
        &'a self,
        extra_tasks: &[&'a dyn Task],
        level: Level,
        message: &str,
    ) -> Result<()> {
        self.publish(self.make_broadcast_statuses(extra_tasks, level, message))
    }

    /// Publish a vector of diagnostic statuses to the diagnostics topic.
    #[inline]
    pub fn publish(&self, status: Vec<DiagnosticStatus>) -> Result<()> {
        let message = DiagnosticArray {
            header: Header::default(),
            status,
        };
        self.publisher.send(message)
    }

    fn map_over_tasks<F>(&self, extra_tasks: &[&'a dyn Task], handler: F) -> Vec<DiagnosticStatus>
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

fn get_diagnostic_period() -> Option<f64> {
    let diag_per = rosrust::param("~diagnostic_period")?;
    println!("FOO");
    let val = diag_per.get().ok()?;
    println!("BAR {}", val);
    Some(val)
}

/// A set of methods for low level handling of the updater.
///
/// These methods should only be used if you know what you are doing and need a more fine grained
/// control of publishing diagnostics.
pub trait UpdaterLowLevelExt {
    /// Generate diagnostic statuses for tasks without publishing the results.
    fn make_update_statuses(&self, extra_tasks: &[&dyn Task]) -> Vec<DiagnosticStatus>;

    /// Generate the diagnostic status for a single task without publishing the results.
    fn make_update_status(&self, task: &dyn Task) -> DiagnosticStatus;

    /// Generate broadcast statuses for tasks without publishing the results.
    fn make_broadcast_statuses(
        &self,
        extra_tasks: &[&dyn Task],
        level: Level,
        message: &str,
    ) -> Vec<DiagnosticStatus>;

    /// Generate the broadcast status for a single task without publishing the results.
    fn make_broadcast_status_for(
        &self,
        task: &dyn Task,
        level: Level,
        message: &str,
    ) -> DiagnosticStatus;
}

impl<'a> UpdaterLowLevelExt for Updater<'a> {
    /// Generate diagnostic statuses for tasks without publishing the results.
    #[inline]
    fn make_update_statuses(&self, extra_tasks: &[&dyn Task]) -> Vec<DiagnosticStatus> {
        self.map_over_tasks(extra_tasks, |task| self.make_update_status(task))
    }

    /// Generate the diagnostic status for a single task without publishing the results.
    fn make_update_status(&self, task: &dyn Task) -> DiagnosticStatus {
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

    /// Generate broadcast statuses for tasks without publishing the results.
    #[inline]
    fn make_broadcast_statuses(
        &self,
        extra_tasks: &[&dyn Task],
        level: Level,
        message: &str,
    ) -> Vec<DiagnosticStatus> {
        self.map_over_tasks(extra_tasks, |task| {
            self.make_broadcast_status_for(task, level, message)
        })
    }

    /// Generate the broadcast status for a single task without publishing the results.
    #[inline]
    fn make_broadcast_status_for(
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
}

static FAILED_TO_LOCK: &'static str = "Failed to acquire lock";
