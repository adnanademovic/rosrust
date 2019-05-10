use crate::msg::actionlib_msgs::{GoalID, GoalStatusArray};
use crate::status_tracker::StatusTracker;
use crate::Action;
use std::sync::{Arc, Mutex};

pub struct ActionServer<T: Action> {
    result_pub: rosrust::Publisher<T::Result>,
    feedback_pub: rosrust::Publisher<T::Feedback>,
    goal_sub: rosrust::Subscriber,
    cancel_sub: rosrust::Subscriber,
    status_frequency: f64,
    status_list: Arc<Mutex<StatusList<T>>>,
    status_timer: std::thread::JoinHandle<()>,
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

fn create_status_publisher<F: Fn()>(frequency: f64, callback: F) -> impl Fn() {
    move || {
        let mut rate = rosrust::rate(frequency);
        rosrust::ros_debug!("Starting timer");
        while rosrust::is_ok() {
            rate.sleep();
            callback()
        }
    }
}

impl<T: Action> ActionServer<T> {
    pub fn new<Fg, Fc, Fs>(
        namespace: &str,
        on_goal: Fg,
        on_cancel: Fc,
    ) -> rosrust::error::Result<Self>
    where
        Fg: Fn(T::Goal) + Send + 'static,
        Fc: Fn(GoalID) + Send + 'static,
    {
        let pub_queue_size = decode_queue_size("actionlib_server_pub_queue_size", 50);
        let sub_queue_size = decode_queue_size("actionlib_server_sub_queue_size", 0);

        let status_pub = rosrust::publish(&format!("{}/status", namespace), pub_queue_size)?;
        let result_pub = rosrust::publish(&format!("{}/result", namespace), pub_queue_size)?;
        let feedback_pub = rosrust::publish(&format!("{}/feedback", namespace), pub_queue_size)?;

        let goal_sub = rosrust::subscribe(&format!("{}/goal", namespace), sub_queue_size, on_goal)?;
        let cancel_sub =
            rosrust::subscribe(&format!("{}/cancel", namespace), sub_queue_size, on_cancel)?;

        let status_frequency = get_status_frequency().unwrap_or(5.0);

        let status_list_timeout = rosrust::param(&format!("{}/status_list_timeout", namespace))
            .ok_or_else(|| "Bad actionlib namespace")?
            .get()
            .unwrap_or(5.0);
        let status_list_timeout =
            rosrust::Duration::from_nanos((status_list_timeout as i64) * 1_000_000_000);

        let status_list = Arc::new(Mutex::new(StatusList {
            publisher: status_pub,
            timeout: status_list_timeout,
            items: vec![],
        }));

        let on_status = {
            let status_list = Arc::clone(&status_list);
            move || {
                if let Err(err) = status_list.lock().expect("Failed to lock mutex").publish() {
                    rosrust::ros_err!("Failed to publish status: {}", err);
                }
            }
        };

        let status_timer = std::thread::spawn(create_status_publisher(status_frequency, on_status));

        Ok(Self {
            result_pub,
            feedback_pub,
            goal_sub,
            cancel_sub,
            status_frequency,
            status_list,
            status_timer,
        })
    }
}

struct StatusList<T> {
    publisher: rosrust::Publisher<GoalStatusArray>,
    timeout: rosrust::Duration,
    items: Vec<StatusTracker<T>>,
}

impl<T: Action> StatusList<T> {
    fn get_status_array(&mut self) -> GoalStatusArray {
        GoalStatusArray::default()
    }

    fn publish(&mut self) -> rosrust::error::Result<()> {
        let status_array = self.get_status_array();
        if !rosrust::is_ok() {
            return Ok(());
        }
        self.publisher.send(status_array)
    }
}

static UNEXPECTED_FAILED_NAME_RESOLVE: &str = "Resolving this parameter name should never fail";
