use crate::msg::actionlib_msgs::{GoalID, GoalStatusArray};
use crate::Action;

pub struct ActionServer<T: Action> {
    status_pub: rosrust::Publisher<GoalStatusArray>,
    result_pub: rosrust::Publisher<T::Result>,
    feedback_pub: rosrust::Publisher<T::Feedback>,
    goal_sub: rosrust::Subscriber,
    cancel_sub: rosrust::Subscriber,
    status_frequency: f64,
    status_list_timeout: rosrust::Duration,
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

impl<T: Action> ActionServer<T> {
    pub fn new(namespace: &str) -> rosrust::error::Result<Self> {
        let pub_queue_size = decode_queue_size("actionlib_server_pub_queue_size", 50);
        let sub_queue_size = decode_queue_size("actionlib_server_sub_queue_size", 0);

        let status_pub = rosrust::publish(&format!("{}/status", namespace), pub_queue_size)?;
        let result_pub = rosrust::publish(&format!("{}/result", namespace), pub_queue_size)?;
        let feedback_pub = rosrust::publish(&format!("{}/feedback", namespace), pub_queue_size)?;

        let goal_sub = rosrust::subscribe(
            &format!("{}/goal", namespace),
            sub_queue_size,
            |_: T::Goal| {
                unimplemented!();
            },
        )?;
        let cancel_sub = rosrust::subscribe(
            &format!("{}/cancel", namespace),
            sub_queue_size,
            |_: GoalID| {
                unimplemented!();
            },
        )?;

        let status_frequency = get_status_frequency().unwrap_or(5.0);

        let status_list_timeout = rosrust::param(&format!("{}/status_list_timeout", namespace))
            .ok_or_else(|| "Bad actionlib namespace")?
            .get()
            .unwrap_or(5.0);
        let status_list_timeout =
            rosrust::Duration::from_nanos((status_list_timeout as i64) * 1_000_000_000);

        let status_timer = std::thread::spawn(|| {
            unimplemented!();
        });

        Ok(Self {
            status_pub,
            result_pub,
            feedback_pub,
            goal_sub,
            cancel_sub,
            status_frequency,
            status_list_timeout,
            status_timer,
        })
    }
}

static UNEXPECTED_FAILED_NAME_RESOLVE: &str = "Resolving this parameter name should never fail";
