pub use self::client_goal_handle::ClientGoalHandle;
pub use self::comm_state_machine::State;
use crate::msg::actionlib_msgs;
use crate::static_messages::{MUTEX_LOCK_FAIL, UNEXPECTED_FAILED_NAME_RESOLVE};
use crate::{Action, ActionResponse, FeedbackBody, GoalBody};
use rosrust::error::Result;
use std::sync::{Arc, Mutex};

mod client_goal_handle;
mod comm_state_machine;
mod goal_manager;

pub struct ActionClient<T: Action> {
    namespace: String,
    pub_goal: rosrust::Publisher<T::Goal>,
    pub_cancel: rosrust::Publisher<actionlib_msgs::GoalID>,
    manager: Arc<Mutex<goal_manager::GoalManager<T>>>,
    last_status_message: Arc<Mutex<Option<actionlib_msgs::GoalStatusArray>>>,
    _status_sub: rosrust::Subscriber,
    _result_sub: rosrust::Subscriber,
    _feedback_sub: rosrust::Subscriber,
}

impl<T: Action> ActionClient<T> {
    pub fn new(namespace: &str) -> Result<Self> {
        let pub_queue_size = decode_queue_size("actionlib_client_pub_queue_size", 10);
        let sub_queue_size = decode_queue_size("actionlib_client_sub_queue_size", 0);

        let pub_goal = rosrust::publish(&format!("{}/goal", namespace), pub_queue_size)?;
        let pub_cancel = rosrust::publish(&format!("{}/cancel", namespace), pub_queue_size)?;

        let on_send_goal = {
            let publisher = pub_goal.clone();
            move |data| {
                if let Err(err) = publisher.send(data) {
                    rosrust::ros_err!("Failed to publish goal: {}", err);
                }
            }
        };

        let on_cancel = {
            let publisher = pub_cancel.clone();
            move |data| {
                if let Err(err) = publisher.send(data) {
                    rosrust::ros_err!("Failed to publish cancel: {}", err);
                }
            }
        };

        let manager = Arc::new(Mutex::new(goal_manager::GoalManager::new(
            on_send_goal,
            on_cancel,
        )));

        let last_status_message = Arc::new(Mutex::new(None));

        let on_status = {
            let last_status_message = Arc::clone(&last_status_message);
            let manager = Arc::clone(&manager);
            move |message: actionlib_msgs::GoalStatusArray| {
                (*last_status_message.lock().expect(MUTEX_LOCK_FAIL)) = Some(message.clone());
                manager
                    .lock()
                    .expect(MUTEX_LOCK_FAIL)
                    .update_statuses(&message);
            }
        };
        let on_result = {
            let manager = Arc::clone(&manager);
            move |result: T::Result| {
                manager
                    .lock()
                    .expect(MUTEX_LOCK_FAIL)
                    .update_results(&result.into_response());
            }
        };
        let on_feedback = {
            let manager = Arc::clone(&manager);
            move |feedback: T::Feedback| {
                manager
                    .lock()
                    .expect(MUTEX_LOCK_FAIL)
                    .update_feedbacks(&feedback.into_response());
            }
        };

        let status_sub =
            rosrust::subscribe(&format!("{}/status", namespace), sub_queue_size, on_status)?;
        let result_sub =
            rosrust::subscribe(&format!("{}/result", namespace), sub_queue_size, on_result)?;
        let feedback_sub = rosrust::subscribe(
            &format!("{}/feedback", namespace),
            sub_queue_size,
            on_feedback,
        )?;

        Ok(Self {
            namespace: namespace.into(),
            pub_goal,
            pub_cancel,
            manager,
            last_status_message,
            _status_sub: status_sub,
            _result_sub: result_sub,
            _feedback_sub: feedback_sub,
        })
    }

    // TODO: make more ergonomic, probably with a builder
    #[inline]
    pub fn send_goal<Ft, Ff>(
        &self,
        goal: GoalBody<T>,
        on_transition: Option<Ft>,
        on_feedback: Option<Ff>,
    ) -> ClientGoalHandle<T>
    where
        Ft: Fn(ClientGoalHandle<T>) + Send + Sync + 'static,
        Ff: Fn(ClientGoalHandle<T>, FeedbackBody<T>) + Send + Sync + 'static,
    {
        self.manager
            .lock()
            .expect(MUTEX_LOCK_FAIL)
            .init_goal(goal, on_transition, on_feedback)
    }

    #[inline]
    pub fn build_goal_sender<'a>(
        &'a self,
        goal: GoalBody<T>,
    ) -> SendGoalBuilder<
        'a,
        T,
        impl Fn(ClientGoalHandle<T>) + Send + Sync + 'static,
        impl Fn(ClientGoalHandle<T>, FeedbackBody<T>) + Send + Sync + 'static,
    > {
        SendGoalBuilder::new(self, goal, |_| {}, |_, _| {})
    }

    #[inline]
    pub fn cancel_all_goals(&self) -> rosrust::error::Result<()> {
        self.cancel_goals_at_and_before_time(rosrust::Time::new())
    }

    #[inline]
    pub fn cancel_goals_at_and_before_time(
        &self,
        stamp: rosrust::Time,
    ) -> rosrust::error::Result<()> {
        self.pub_cancel.send(actionlib_msgs::GoalID {
            id: String::new(),
            stamp,
        })
    }

    /*
    TODO: this needs certain pub/sub internals present to be implemented
    pub fn wait_for_server(&self, timeout: Option<rosrust::Duration>) -> bool {
    }
    */
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

pub struct SendGoalBuilder<'a, T: Action, Ft, Ff> {
    client: &'a ActionClient<T>,
    goal: GoalBody<T>,
    on_transition: Option<Ft>,
    on_feedback: Option<Ff>,
}

impl<'a, T: Action, Ft, Ff> SendGoalBuilder<'a, T, Ft, Ff>
where
    Ft: Fn(ClientGoalHandle<T>) + Send + Sync + 'static,
    Ff: Fn(ClientGoalHandle<T>, FeedbackBody<T>) + Send + Sync + 'static,
{
    fn new(client: &'a ActionClient<T>, goal: GoalBody<T>, _: Ft, _: Ff) -> Self {
        Self {
            client,
            goal,
            on_transition: None,
            on_feedback: None,
        }
    }

    #[inline]
    pub fn on_transition<Fnew>(self, callback: Fnew) -> SendGoalBuilder<'a, T, Fnew, Ff>
    where
        Fnew: Fn(ClientGoalHandle<T>) + Send + Sync + 'static,
    {
        SendGoalBuilder {
            client: self.client,
            goal: self.goal,
            on_transition: Some(callback),
            on_feedback: self.on_feedback,
        }
    }

    #[inline]
    pub fn on_feedback<Fnew>(self, callback: Fnew) -> SendGoalBuilder<'a, T, Ft, Fnew>
    where
        Fnew: Fn(ClientGoalHandle<T>, FeedbackBody<T>) + Send + Sync + 'static,
    {
        SendGoalBuilder {
            client: self.client,
            goal: self.goal,
            on_transition: self.on_transition,
            on_feedback: Some(callback),
        }
    }

    #[inline]
    pub fn send(self) -> ClientGoalHandle<T> {
        self.client
            .send_goal(self.goal, self.on_transition, self.on_feedback)
    }
}
