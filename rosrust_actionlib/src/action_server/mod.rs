pub use self::server_goal_handle::{ServerGoalHandle, ServerGoalHandleMessageBuilder};
use self::status_list::StatusList;
use self::status_tracker::StatusTracker;
use crate::static_messages::{MUTEX_LOCK_FAIL, UNEXPECTED_FAILED_NAME_RESOLVE};
use crate::{
    Action, ActionGoal, ActionResponse, FeedbackBody, GoalBody, GoalID, GoalStatus, Response,
    ResultBody,
};
use rosrust::error::Result;
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;

mod goal_coordinator;
mod server_goal_handle;
mod status_list;
mod status_tracker;

pub struct ActionServer<T: Action> {
    result_pub: rosrust::Publisher<T::Result>,
    feedback_pub: rosrust::Publisher<T::Feedback>,
    status_list: Arc<Mutex<StatusList<T>>>,
    is_alive: Arc<AtomicBool>,
    status_frequency: f64,
    _goal_sub: rosrust::Subscriber,
    _cancel_sub: rosrust::Subscriber,
}

impl<T: Action> Drop for ActionServer<T> {
    fn drop(&mut self) {
        self.is_alive.store(false, Ordering::Relaxed);
    }
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

fn create_status_publisher<T: Action>(
    frequency: f64,
    status_list: Arc<Mutex<StatusList<T>>>,
    is_alive: Arc<AtomicBool>,
) -> impl Fn() {
    move || {
        let rate = rosrust::rate(frequency);
        rosrust::ros_debug!("Starting timer");
        while rosrust::is_ok() && is_alive.load(Ordering::Relaxed) {
            rate.sleep();
            if let Err(err) = status_list.lock().expect(MUTEX_LOCK_FAIL).publish() {
                rosrust::ros_err!("Failed to publish status: {}", err);
            }
        }
    }
}

impl<T: Action> ActionServer<T> {
    pub fn new(
        namespace: &str,
        on_goal: ActionServerOnRequest<T>,
        on_cancel: ActionServerOnRequest<T>,
    ) -> Result<Self> {
        let pub_queue_size = decode_queue_size("actionlib_server_pub_queue_size", 50);
        let sub_queue_size = decode_queue_size("actionlib_server_sub_queue_size", 0);

        let status_pub = rosrust::publish(&format!("{}/status", namespace), pub_queue_size)?;
        let result_pub = rosrust::publish(&format!("{}/result", namespace), pub_queue_size)?;
        let feedback_pub = rosrust::publish(&format!("{}/feedback", namespace), pub_queue_size)?;

        let status_frequency = get_status_frequency().unwrap_or(5.0);

        let status_list_timeout = rosrust::param(&format!("{}/status_list_timeout", namespace))
            .ok_or("Bad actionlib namespace")?
            .get()
            .unwrap_or(5.0);
        let status_list_timeout = (status_list_timeout * 1_000_000_000.0) as i64;

        let status_list = Arc::new(Mutex::new(StatusList::new(status_list_timeout, status_pub)));

        let goal_coordinator = Arc::new(goal_coordinator::GoalCoordinator::new(
            result_pub.clone(),
            feedback_pub.clone(),
            on_goal,
            on_cancel,
            Arc::clone(&status_list),
        ));

        let is_alive = Arc::new(AtomicBool::new(true));

        thread::spawn(create_status_publisher(
            status_frequency,
            Arc::clone(&status_list),
            Arc::clone(&is_alive),
        ));

        let internal_on_goal = {
            let goal_coordinator = Arc::clone(&goal_coordinator);
            move |goal| {
                if let Err(err) = goal_coordinator.handle_on_goal(T::Goal::into_goal(goal)) {
                    rosrust::ros_err!("Failed to handle goal creation: {}", err);
                }
            }
        };

        let goal_sub = rosrust::subscribe(
            &format!("{}/goal", namespace),
            sub_queue_size,
            internal_on_goal,
        )?;

        let internal_on_cancel = move |goal_id: GoalID| {
            if let Err(err) = goal_coordinator.handle_on_cancel(goal_id) {
                rosrust::ros_err!("Failed to handle goal cancellation: {}", err);
            }
        };

        let cancel_sub = rosrust::subscribe(
            &format!("{}/cancel", namespace),
            sub_queue_size,
            internal_on_cancel,
        )?;

        let action_server = Self {
            result_pub,
            feedback_pub,
            status_list,
            is_alive,
            status_frequency,
            _goal_sub: goal_sub,
            _cancel_sub: cancel_sub,
        };

        action_server.publish_status()?;

        Ok(action_server)
    }

    pub fn new_simple<F>(namespace: &str, handler: F) -> Result<Self>
    where
        F: Fn(ServerSimpleGoalHandle<T>) + Send + Sync + 'static,
    {
        let active_goals = Arc::new(Mutex::new(HashMap::new()));

        let on_goal = {
            let active_goals = Arc::clone(&active_goals);
            let handler = Arc::new(handler);

            move |server_goal_handle: ServerGoalHandle<T>| {
                let id = server_goal_handle.goal_id().id;

                let canceled = Arc::new(AtomicBool::new(false));

                active_goals
                    .lock()
                    .expect(MUTEX_LOCK_FAIL)
                    .insert(id.clone(), Arc::clone(&canceled));

                {
                    let handler = Arc::clone(&handler);
                    let active_goals = Arc::clone(&active_goals);

                    thread::spawn(move || {
                        let goal_handle = ServerSimpleGoalHandle {
                            goal_handle: server_goal_handle,
                            canceled,
                        };
                        goal_handle
                            .response()
                            .text("This goal has been accepted by the simple action server")
                            .send_accepted();
                        handler(goal_handle);
                        active_goals.lock().expect(MUTEX_LOCK_FAIL).remove(&id);
                    });
                }

                Ok(())
            }
        };

        let on_cancel = {
            let active_goals = Arc::clone(&active_goals);
            move |server_goal_handle: ServerGoalHandle<T>| {
                let id = server_goal_handle.goal_id().id;
                if let Some(flag) = active_goals.lock().expect(MUTEX_LOCK_FAIL).remove(&id) {
                    flag.store(true, Ordering::SeqCst);
                }
                Ok(())
            }
        };

        Self::new(namespace, Box::new(on_goal), Box::new(on_cancel))
    }

    #[inline]
    pub fn status_frequency(&self) -> f64 {
        self.status_frequency
    }

    #[inline]
    pub fn publish_feedback(&self, status: GoalStatus, body: FeedbackBody<T>) -> Result<()> {
        publish_response(&self.feedback_pub, status, body)
    }

    #[inline]
    pub fn publish_result(&self, status: GoalStatus, body: ResultBody<T>) -> Result<()> {
        publish_response(&self.result_pub, status, body)
    }

    #[inline]
    pub fn publish_status(&self) -> Result<()> {
        self.status_list.lock().expect(MUTEX_LOCK_FAIL).publish()
    }
}

pub struct ServerSimpleGoalHandle<T: Action> {
    goal_handle: ServerGoalHandle<T>,
    canceled: Arc<AtomicBool>,
}

impl<T: Action> ServerSimpleGoalHandle<T> {
    pub fn response(&self) -> ServerGoalHandleMessageBuilder<T> {
        self.goal_handle.response()
    }

    pub fn handle(&self) -> &ServerGoalHandle<T> {
        &self.goal_handle
    }

    pub fn handle_mut(&mut self) -> &ServerGoalHandle<T> {
        &self.goal_handle
    }

    pub fn goal(&self) -> &GoalBody<T> {
        self.goal_handle.goal()
    }

    pub fn canceled(&self) -> bool {
        self.canceled.load(Ordering::SeqCst)
    }
}

pub type ActionServerOnRequest<T> = Box<dyn Fn(ServerGoalHandle<T>) -> Result<()> + Send + Sync>;

fn publish_response<T: ActionResponse>(
    publisher: &rosrust::Publisher<T>,
    status: GoalStatus,
    body: T::Body,
) -> Result<()> {
    let mut action_response = Response {
        header: Default::default(),
        status,
        body,
    };
    if !rosrust::is_ok() {
        return Ok(());
    }
    action_response.header.stamp = rosrust::now();
    publisher.send(T::from_response(action_response))
}
