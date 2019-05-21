use crate::action_client::State;
use crate::static_messages::MUTEX_LOCK_FAIL;
use crate::{
    Action, ActionClient, AsyncClientGoalHandle, FeedbackBody, GoalBody, GoalState, ResultBody,
    SyncClientGoalHandle,
};
use rosrust::error::Result;
use rosrust::{Duration, Time};
use std::sync::{Arc, Condvar, Mutex};

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum SimpleGoalState {
    Pending,
    Active,
    Done,
}

type DoneCondition = (Mutex<()>, Condvar);

#[allow(dead_code)]
pub struct SimpleActionClient<T: Action> {
    action_client: ActionClient<T>,
    goal_handle: Option<AsyncClientGoalHandle<T>>,
    callback_handle: Option<Arc<Mutex<CallbackStatus<T>>>>,
    done_condition: Arc<DoneCondition>,
    simple_state: Arc<Mutex<SimpleGoalState>>,
}

impl<T: Action> SimpleActionClient<T> {
    pub fn new(namespace: &str) -> Result<Self> {
        Ok(Self {
            action_client: ActionClient::new(namespace)?,
            goal_handle: None,
            callback_handle: None,
            done_condition: Arc::new((Mutex::new(()), Condvar::new())),
            simple_state: Arc::new(Mutex::new(SimpleGoalState::Done)),
        })
    }

    pub fn wait_for_server(&self, timeout: Option<Duration>) -> bool {
        if let Some(timeout) = timeout {
            self.action_client.wait_for_server(timeout)
        } else {
            self.action_client.wait_for_server_forever();
            true
        }
    }

    pub fn build_goal_sender<'a>(&'a mut self, goal: GoalBody<T>) -> SendGoalBuilder<'a, T> {
        SendGoalBuilder::new(self, goal)
    }

    pub fn send_goal(
        &mut self,
        goal: GoalBody<T>,
        on_done: Option<CallbackStatusOnDone<T>>,
        on_active: Option<CallbackStatusOnActive>,
        on_feedback: Option<CallbackStatusOnFeedback<T>>,
    ) {
        self.stop_tracking_goal();

        self.simple_state = Arc::new(Mutex::new(SimpleGoalState::Pending));

        let callback_handle = Arc::new(Mutex::new(CallbackStatus {
            expired: false,
            namespace: self.action_client.namespace().into(),
            state: Arc::clone(&self.simple_state),
            on_done,
            on_active,
            on_feedback,
            done_condition: Arc::clone(&self.done_condition),
        }));

        let handle_transition = CallbackStatus::create_transition_handler(&callback_handle);
        let handle_feedback = CallbackStatus::create_feedback_handler(&callback_handle);

        self.callback_handle = Some(callback_handle);

        let goal_handle = self
            .action_client
            .build_goal_sender(goal)
            .on_transition(handle_transition)
            .on_feedback(handle_feedback)
            .send();

        self.goal_handle = Some(goal_handle);
    }

    pub fn send_goal_and_wait(
        &mut self,
        goal: GoalBody<T>,
        execute_timeout: Option<Duration>,
        preempt_timeout: Option<Duration>,
    ) -> GoalState {
        self.build_goal_sender(goal).send();

        if self.wait_for_result(execute_timeout) {
            return self.state();
        }

        rosrust::ros_debug!("Canceling goal");
        self.cancel_goal();
        let timeout_time = preempt_timeout
            .as_ref()
            .map(Duration::seconds)
            .unwrap_or_default();
        let finished = self.wait_for_result(preempt_timeout);
        rosrust::ros_debug!(
            "Preempt {} within specified preempt_timeout [{}]",
            if finished {
                "finished"
            } else {
                "didn't finish"
            },
            timeout_time
        );

        self.state()
    }

    pub fn wait_for_result(&self, timeout: Option<Duration>) -> bool {
        if self.goal_handle.is_none() {
            rosrust::ros_err!("Called wait_for_result when no goal exists");
            return false;
        }
        let timeout_time = timeout.map(|timeout| (rosrust::now() + timeout).seconds());
        let loop_period_seconds = 0.1;

        let (ref lock, ref condition) = *self.done_condition;
        let mut condition_lock_guard = lock.lock().expect(MUTEX_LOCK_FAIL);
        while rosrust::is_ok() {
            let time_left = match timeout_time {
                None => loop_period_seconds,
                Some(tt) => (tt - rosrust::now().seconds()).min(loop_period_seconds),
            };

            if self.simple_state() == SimpleGoalState::Done {
                return true;
            }

            if time_left < 0.0 {
                return false;
            }

            condition_lock_guard = condition
                .wait_timeout(
                    condition_lock_guard,
                    std::time::Duration::from_millis((time_left * 1000.0) as u64),
                )
                .expect(MUTEX_LOCK_FAIL)
                .0;
        }

        false
    }

    fn simple_state(&self) -> SimpleGoalState {
        *self.simple_state.lock().expect(MUTEX_LOCK_FAIL)
    }

    pub fn result(&self) -> Option<ResultBody<T>> {
        let result = self
            .goal_handle
            .as_ref()
            .and_then(AsyncClientGoalHandle::result);
        if result.is_none() {
            rosrust::ros_err!("Called result when no goal is running");
        }
        result
    }

    pub fn state(&self) -> GoalState {
        let inner_goal_state = self
            .goal_handle
            .as_ref()
            .map(AsyncClientGoalHandle::goal_state)
            .unwrap_or(GoalState::Lost);
        match inner_goal_state {
            GoalState::Recalling => GoalState::Pending,
            GoalState::Preempting => GoalState::Active,
            other => other,
        }
    }

    pub fn goal_status_text(&self) -> Option<String> {
        let status_text = self
            .goal_handle
            .as_ref()
            .map(AsyncClientGoalHandle::goal_status_text);
        if status_text.is_none() {
            rosrust::ros_err!("Called goal_status_text when no goal is running");
        }
        status_text
    }

    pub fn cancel_all_goals(&self) -> Result<()> {
        self.action_client.cancel_all_goals()
    }

    pub fn cancel_goals_at_and_before_time(&self, time: Time) -> Result<()> {
        self.action_client.cancel_goals_at_and_before_time(time)
    }

    pub fn cancel_goal(&self) {
        if let Some(ref gh) = self.goal_handle {
            gh.cancel();
        }
    }

    pub fn stop_tracking_goal(&mut self) {
        if let Some(ref cb_handle) = self.callback_handle {
            cb_handle.lock().expect(MUTEX_LOCK_FAIL).expired = true;
        }
        self.callback_handle = None;
        self.goal_handle = None;
    }
}

type CallbackStatusOnDone<T> = Box<dyn Fn(GoalState, Option<ResultBody<T>>) + Send>;
type CallbackStatusOnActive = Box<dyn Fn() + Send>;
type CallbackStatusOnFeedback<T> = Box<dyn Fn(FeedbackBody<T>) + Send>;

#[allow(dead_code)]
struct CallbackStatus<T: Action> {
    expired: bool,
    namespace: String,
    state: Arc<Mutex<SimpleGoalState>>,
    on_done: Option<CallbackStatusOnDone<T>>,
    on_active: Option<CallbackStatusOnActive>,
    on_feedback: Option<CallbackStatusOnFeedback<T>>,
    done_condition: Arc<DoneCondition>,
}

impl<T: Action> CallbackStatus<T> {
    fn handle_transition(&mut self, gh: SyncClientGoalHandle<T>) {
        let comm_state = gh.comm_state();

        match (comm_state, self.state()) {
            (State::Active, SimpleGoalState::Done)
            | (State::Recalling, SimpleGoalState::Active)
            | (State::Recalling, SimpleGoalState::Done)
            | (State::Preempting, SimpleGoalState::Done) => {
                rosrust::ros_err!("Received comm state {:?} when in simple state {:?} with SimpleActionClient in NS {}", comm_state, self.state, self.namespace);
            }
            (State::Done, SimpleGoalState::Done) => {
                rosrust::ros_err!("SimpleActionClient received {:?} twice", comm_state);
            }
            (State::Active, SimpleGoalState::Pending)
            | (State::Preempting, SimpleGoalState::Pending) => {
                self.set_state(SimpleGoalState::Active);
                if let Some(ref on_active) = self.on_active {
                    (*on_active)();
                }
            }
            (State::Done, SimpleGoalState::Pending) | (State::Done, SimpleGoalState::Active) => {
                self.set_state(SimpleGoalState::Done);
                if let Some(ref on_done) = self.on_done {
                    (*on_done)(gh.goal_state(), gh.result());
                }
                let (ref lock, ref condition) = *self.done_condition;
                let mut _condition_lock_guard = lock.lock().expect(MUTEX_LOCK_FAIL);
                condition.notify_all();
            }
            _ => {}
        }
    }

    fn state(&self) -> SimpleGoalState {
        *self.state.lock().expect(MUTEX_LOCK_FAIL)
    }

    fn set_state(&mut self, value: SimpleGoalState) {
        *self.state.lock().expect(MUTEX_LOCK_FAIL) = value;
    }

    fn handle_feedback(&mut self, _gh: SyncClientGoalHandle<T>, feedback: FeedbackBody<T>) {
        if self.expired {
            return;
        }
        if let Some(ref on_feedback) = self.on_feedback {
            (*on_feedback)(feedback);
        }
    }

    fn create_transition_handler(
        callback_status: &Arc<Mutex<Self>>,
    ) -> impl for<'a> Fn(SyncClientGoalHandle<'a, T>) {
        let callback_status = Arc::clone(callback_status);
        move |gh| {
            callback_status
                .lock()
                .expect(MUTEX_LOCK_FAIL)
                .handle_transition(gh)
        }
    }

    fn create_feedback_handler(
        callback_status: &Arc<Mutex<Self>>,
    ) -> impl for<'a> Fn(SyncClientGoalHandle<'a, T>, FeedbackBody<T>) {
        let callback_status = Arc::clone(callback_status);
        move |gh, fb| {
            callback_status
                .lock()
                .expect(MUTEX_LOCK_FAIL)
                .handle_feedback(gh, fb)
        }
    }
}

pub struct SendGoalBuilder<'a, T: Action> {
    client: &'a mut SimpleActionClient<T>,
    goal: GoalBody<T>,
    on_done: Option<CallbackStatusOnDone<T>>,
    on_active: Option<CallbackStatusOnActive>,
    on_feedback: Option<CallbackStatusOnFeedback<T>>,
}

impl<'a, T: Action> SendGoalBuilder<'a, T> {
    fn new(client: &'a mut SimpleActionClient<T>, goal: GoalBody<T>) -> Self {
        Self {
            client,
            goal,
            on_done: None,
            on_active: None,
            on_feedback: None,
        }
    }

    #[inline]
    pub fn on_done<Fnew>(mut self, callback: Fnew) -> Self
    where
        Fnew: Fn(GoalState, Option<ResultBody<T>>) + Send + 'static,
    {
        self.on_done = Some(Box::new(callback));
        self
    }

    #[inline]
    pub fn on_active<Fnew>(mut self, callback: Fnew) -> Self
    where
        Fnew: Fn() + Send + 'static,
    {
        self.on_active = Some(Box::new(callback));
        self
    }

    #[inline]
    pub fn on_feedback<Fnew>(mut self, callback: Fnew) -> Self
    where
        Fnew: Fn(FeedbackBody<T>) + Send + 'static,
    {
        self.on_feedback = Some(Box::new(callback));
        self
    }

    #[inline]
    pub fn send(self) {
        self.client
            .send_goal(self.goal, self.on_done, self.on_active, self.on_feedback)
    }
}
