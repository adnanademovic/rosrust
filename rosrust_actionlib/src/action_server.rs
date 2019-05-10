use crate::msg::actionlib_msgs::{GoalID, GoalStatusArray};
use crate::status_tracker::StatusTracker;
use crate::Action;
use std::sync::{Arc, Mutex};

pub struct ActionServer<T: Action> {
    fields: Arc<Mutex<Fields<T>>>,
    _goal_sub: rosrust::Subscriber,
    _cancel_sub: rosrust::Subscriber,
    _status_timer: std::thread::JoinHandle<()>,
}

struct Fields<T: Action> {
    result_pub: rosrust::Publisher<T::Result>,
    feedback_pub: rosrust::Publisher<T::Feedback>,
    status_pub: rosrust::Publisher<GoalStatusArray>,
    status_list_timeout: i64,
    status_list: Vec<StatusTracker<T>>,
    status_frequency: f64,
    on_goal: Box<Fn(T::Goal) + Send + Sync>,
    on_cancel: Box<Fn(GoalID) + Send + Sync>,
}

fn decode_queue_size(param: &str, default: usize) -> usize {
    let param: Option<i32> = rosrust::param(param)
        .expect(UNEXPECTED_FAILED_NAME_RESOLVE)
        .get()
        .ok();
    match param {
        None => default,
        Some(v) if v < 0 => default,
        Some(v) => v as usize,
    }
}

fn get_status_frequency() -> Option<f64> {
    let key = rosrust::param("actionlib_status_frequency")
        .expect(UNEXPECTED_FAILED_NAME_RESOLVE)
        .search()
        .ok()?;
    rosrust::param(&key)?.get().ok()
}

fn create_status_publisher<F>(frequency: f64, callback: F) -> impl Fn()
where
    F: Fn() -> rosrust::error::Result<()>,
{
    move || {
        let mut rate = rosrust::rate(frequency);
        rosrust::ros_debug!("Starting timer");
        while rosrust::is_ok() {
            rate.sleep();
            if let Err(err) = callback() {
                rosrust::ros_err!("Failed to publish status: {}", err);
            }
        }
    }
}

impl<T: Action> ActionServer<T> {
    pub fn new(namespace: &str) -> rosrust::error::Result<Self> {
        let pub_queue_size = decode_queue_size("actionlib_server_pub_queue_size", 50);
        let sub_queue_size = decode_queue_size("actionlib_server_sub_queue_size", 0);

        let status_pub = rosrust::publish(&format!("{}/status", namespace), pub_queue_size)?;
        let result_pub = rosrust::publish(&format!("{}/result", namespace), pub_queue_size)?;
        let feedback_pub = rosrust::publish(&format!("{}/feedback", namespace), pub_queue_size)?;

        let status_frequency = get_status_frequency().unwrap_or(5.0);

        let status_list_timeout = rosrust::param(&format!("{}/status_list_timeout", namespace))
            .ok_or_else(|| "Bad actionlib namespace")?
            .get()
            .unwrap_or(5.0);
        let status_list_timeout = (status_list_timeout * 1_000_000_000.0) as i64;

        let fields = Arc::new(Mutex::new(Fields {
            result_pub,
            feedback_pub,
            status_pub,
            status_frequency,
            status_list: vec![],
            status_list_timeout,
            on_goal: Box::new(|_| {}),
            on_cancel: Box::new(|_| {}),
        }));

        let on_status = {
            let fields = Arc::clone(&fields);
            move || fields.lock().expect(MUTEX_LOCK_FAIL).publish_status()
        };

        let status_timer = std::thread::spawn(create_status_publisher(status_frequency, on_status));

        let internal_on_goal = {
            let fields = Arc::clone(&fields);
            move |goal| fields.lock().expect(MUTEX_LOCK_FAIL).handle_on_goal(goal)
        };

        let goal_sub = rosrust::subscribe(
            &format!("{}/goal", namespace),
            sub_queue_size,
            internal_on_goal,
        )?;

        let internal_on_cancel = {
            let fields = Arc::clone(&fields);
            move |goal_id| {
                fields
                    .lock()
                    .expect(MUTEX_LOCK_FAIL)
                    .handle_on_cancel(goal_id)
            }
        };

        let cancel_sub = rosrust::subscribe(
            &format!("{}/cancel", namespace),
            sub_queue_size,
            internal_on_cancel,
        )?;

        Ok(Self {
            fields,
            _goal_sub: goal_sub,
            _cancel_sub: cancel_sub,
            _status_timer: status_timer,
        })
    }

    pub fn set_on_goal<F>(&mut self, on_goal: F)
    where
        F: Fn(T::Goal) + Send + Sync + 'static,
    {
        self.fields.lock().expect(MUTEX_LOCK_FAIL).on_goal = Box::new(on_goal);
    }

    pub fn set_on_cancel<F>(&mut self, on_cancel: F)
    where
        F: Fn(GoalID) + Send + Sync + 'static,
    {
        self.fields.lock().expect(MUTEX_LOCK_FAIL).on_cancel = Box::new(on_cancel);
    }
}

impl<T: Action> Fields<T> {
    fn get_status_array(&mut self) -> GoalStatusArray {
        let now = rosrust::now();
        let now_nanos = now.nanos();
        let status_list_timeout = self.status_list_timeout;
        self.status_list.retain(|tracker| {
            let destruction_time = tracker.handle_destruction_time.nanos();
            if destruction_time == 0 {
                return true;
            }
            if destruction_time + status_list_timeout > now_nanos {
                return true;
            }
            rosrust::ros_debug!(
                "Item {} with destruction time of {} being removed from list.  Now = {}",
                tracker.status.goal_id.id,
                tracker.handle_destruction_time.seconds(),
                now.seconds()
            );
            return false;
        });
        let status_list = self
            .status_list
            .iter()
            .map(|tracker| tracker.status.clone())
            .collect();
        GoalStatusArray {
            header: Default::default(),
            status_list,
        }
    }

    fn publish_status(&mut self) -> rosrust::error::Result<()> {
        let status_array = self.get_status_array();
        if !rosrust::is_ok() {
            return Ok(());
        }
        self.status_pub.send(status_array)
    }

    fn handle_on_goal(&self, goal: T::Goal) {
        unimplemented!();
        (*self.on_goal)(goal);
    }

    fn handle_on_cancel(&self, goal_id: GoalID) {
        unimplemented!();
        (*self.on_cancel)(goal_id);
    }
}

static UNEXPECTED_FAILED_NAME_RESOLVE: &str = "Resolving this parameter name should never fail";
static MUTEX_LOCK_FAIL: &str = "Failed to lock mutex";
