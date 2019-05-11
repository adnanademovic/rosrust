use crate::msg::actionlib_msgs::{GoalID, GoalStatus, GoalStatusArray};
use crate::status_tracker::StatusTracker;
use crate::{Action, ActionGoal, ActionResponse, Goal, Response};
use rosrust::error::Result;
use std::sync::{Arc, Mutex, Weak};

pub struct ActionServer<T: Action> {
    fields: Arc<Mutex<ActionServerState<T>>>,
    _goal_sub: rosrust::Subscriber,
    _cancel_sub: rosrust::Subscriber,
    _status_timer: std::thread::JoinHandle<()>,
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
    F: Fn() -> Result<()>,
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
    pub fn new(namespace: &str) -> Result<Self> {
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

        let fields = Arc::new(Mutex::new(ActionServerState {
            last_cancel_ns: 0,
            result_pub,
            feedback_pub,
            status_pub,
            status_frequency,
            status_list: vec![],
            status_list_timeout,
            on_goal: Box::new(|_| Ok(())),
            on_cancel: Box::new(|_| Ok(())),
            self_reference: Weak::new(),
        }));
        fields.lock().expect(MUTEX_LOCK_FAIL).self_reference = Arc::downgrade(&fields);

        let on_status = {
            let fields = Arc::clone(&fields);
            move || fields.lock().expect(MUTEX_LOCK_FAIL).publish_status()
        };

        let status_timer = std::thread::spawn(create_status_publisher(status_frequency, on_status));

        let internal_on_goal = {
            let fields = Arc::clone(&fields);
            move |goal| {
                if let Err(err) = fields
                    .lock()
                    .expect(MUTEX_LOCK_FAIL)
                    .handle_on_goal(T::Goal::into_goal(goal))
                {
                    rosrust::ros_err!("Failed to handle goal creation: {}", err);
                }
            }
        };

        let goal_sub = rosrust::subscribe(
            &format!("{}/goal", namespace),
            sub_queue_size,
            internal_on_goal,
        )?;

        let internal_on_cancel = {
            let fields = Arc::clone(&fields);
            move |goal_id| {
                if let Err(err) = fields
                    .lock()
                    .expect(MUTEX_LOCK_FAIL)
                    .handle_on_cancel(goal_id)
                {
                    rosrust::ros_err!("Failed to handle goal creation: {}", err);
                }
            }
        };

        let cancel_sub = rosrust::subscribe(
            &format!("{}/cancel", namespace),
            sub_queue_size,
            internal_on_cancel,
        )?;

        let action_server = Self {
            fields,
            _goal_sub: goal_sub,
            _cancel_sub: cancel_sub,
            _status_timer: status_timer,
        };

        action_server.publish_status()?;

        Ok(action_server)
    }

    #[inline]
    pub fn publish_status(&self) -> Result<()> {
        self.fields.lock().expect(MUTEX_LOCK_FAIL).publish_status()
    }

    #[inline]
    pub fn set_on_goal(&mut self, on_goal: ActionServerOnGoal<T>) {
        self.fields.lock().expect(MUTEX_LOCK_FAIL).on_goal = on_goal;
    }

    #[inline]
    pub fn set_on_cancel(&mut self, on_cancel: ActionServerOnCancel) {
        self.fields.lock().expect(MUTEX_LOCK_FAIL).on_cancel = on_cancel;
    }
}

pub type ActionServerOnGoal<T> = Box<Fn(ServerGoalHandle<T>) -> Result<()> + Send + Sync>;
pub type ActionServerOnCancel = Box<Fn(GoalID) -> Result<()> + Send + Sync>;

struct ActionServerState<T: Action> {
    last_cancel_ns: i64,
    result_pub: rosrust::Publisher<T::Result>,
    feedback_pub: rosrust::Publisher<T::Feedback>,
    status_pub: rosrust::Publisher<GoalStatusArray>,
    status_list_timeout: i64,
    status_list: Vec<StatusTracker<GoalBody<T>>>,
    status_frequency: f64,
    on_goal: ActionServerOnGoal<T>,
    on_cancel: ActionServerOnCancel,
    self_reference: Weak<Mutex<Self>>,
}

impl<T: Action> ActionServerState<T> {
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

    pub fn publish_status(&mut self) -> Result<()> {
        let status_array = self.get_status_array();
        if !rosrust::is_ok() {
            return Ok(());
        }
        self.status_pub.send(status_array)
    }

    pub fn publish_feedback(&self, status: GoalStatus, body: FeedbackBody<T>) -> Result<()> {
        let action_feedback = Response {
            header: Default::default(),
            status,
            body,
        };
        if !rosrust::is_ok() {
            return Ok(());
        }
        self.feedback_pub
            .send(T::Feedback::from_response(action_feedback))
    }

    pub fn publish_result(&self, status: GoalStatus, body: ResultBody<T>) -> Result<()> {
        let action_result = Response {
            header: Default::default(),
            status,
            body,
        };
        if !rosrust::is_ok() {
            return Ok(());
        }
        self.result_pub
            .send(T::Result::from_response(action_result))
    }

    fn handle_on_goal(&mut self, goal: GoalType<T>) -> Result<()> {
        rosrust::ros_debug!("The action server has received a new goal request");

        let goal_id = goal.id.id.clone();

        let existing_tracker = self
            .status_list
            .iter_mut()
            .find(|tracker| goal_id == tracker.status.goal_id.id);

        if let Some(mut tracker) = existing_tracker {
            rosrust::ros_debug!(
                "Goal {} was already in the status list with status {}",
                goal.id.id,
                tracker.status.status
            );

            tracker.handle_destruction_time = rosrust::now();

            if tracker.status.status == GoalStatus::RECALLING {
                tracker.status.status = GoalStatus::RECALLED;
                let status = tracker.status.clone();

                self.publish_result(status, Default::default())?;
            }

            return Ok(());
        }

        let fields = self
            .self_reference
            .upgrade()
            .ok_or_else(|| "Action Server was deconstructed before action was handled")?;

        let tracker = StatusTracker::from_goal(goal);
        let goal_timestamp = tracker.status.goal_id.stamp.nanos();

        self.status_list.push(tracker);

        let goal_handle = ServerGoalHandle { fields, goal_id };

        if goal_timestamp != 0 && goal_timestamp <= self.last_cancel_ns {
            goal_handle.set_canceled(None, "This goal handle was canceled by the action server because its timestamp is before the timestamp of the last cancel request")?;
            return Ok(());
        };

        (*self.on_goal)(goal_handle)
    }

    fn handle_on_cancel(&self, goal_id: GoalID) -> Result<()> {
        unimplemented!();
        (*self.on_cancel)(goal_id)
    }
}

pub struct ServerGoalHandle<T: Action> {
    fields: Arc<Mutex<ActionServerState<T>>>,
    goal_id: String,
}

impl<T: Action> ServerGoalHandle<T> {
    pub fn set_canceled(&self, result: Option<ResultBody<T>>, text: &str) -> Result<()> {
        unimplemented!()
    }
}

static UNEXPECTED_FAILED_NAME_RESOLVE: &str = "Resolving this parameter name should never fail";
static MUTEX_LOCK_FAIL: &str = "Failed to lock mutex";

type GoalBody<T> = <<T as Action>::Goal as ActionGoal>::Body;
type GoalType<T> = Goal<GoalBody<T>>;
type ResultBody<T> = <<T as Action>::Result as ActionResponse>::Body;
type ResultType<T> = Response<ResultBody<T>>;
type FeedbackBody<T> = <<T as Action>::Feedback as ActionResponse>::Body;
type FeedbackType<T> = Response<FeedbackBody<T>>;
