use crate::action_client::State;
use crate::static_messages::MUTEX_LOCK_FAIL;
use crate::{
    Action, ActionClient, ClientGoalHandle, FeedbackBody, GoalBody, GoalState, ResultBody,
};
use rosrust::error::Result;
use rosrust::{Duration, Time};
use std::sync::{Arc, Mutex};

#[derive(Copy, Clone, Debug)]
pub enum SimpleGoalState {
    Pending,
    Active,
    Done,
}

#[allow(dead_code)]
pub struct SimpleActionClient<T: Action> {
    action_client: ActionClient<T>,
    goal_handle: Option<ClientGoalHandle<T>>,
    callback_handle: Option<Arc<Mutex<CallbackStatus<T>>>>,
}

impl<T: Action> SimpleActionClient<T> {
    pub fn new(namespace: &str) -> Result<Self> {
        Ok(Self {
            action_client: ActionClient::new(namespace)?,
            goal_handle: None,
            callback_handle: None,
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

    pub fn build_goal_sender<'a>(
        &'a mut self,
        goal: GoalBody<T>,
    ) -> SendGoalBuilder<
        'a,
        T,
        impl Fn(GoalState, Option<ResultBody<T>>) + Send + 'static,
        impl Fn() + Send + 'static,
        impl Fn(FeedbackBody<T>) + Send + 'static,
    > {
        SendGoalBuilder::new(self, goal, |_, _| {}, || {}, |_| {})
    }

    pub fn send_goal<Fdone, Factive, Ffeedback>(
        &mut self,
        goal: GoalBody<T>,
        on_done: Option<Fdone>,
        on_active: Option<Factive>,
        on_feedback: Option<Ffeedback>,
    ) where
        Fdone: Fn(GoalState, Option<ResultBody<T>>) + Send + 'static,
        Factive: Fn() + Send + 'static,
        Ffeedback: Fn(FeedbackBody<T>) + Send + 'static,
    {
        self.stop_tracking_goal();

        let callback_handle = Arc::new(Mutex::new(CallbackStatus {
            expired: false,
            namespace: self.action_client.namespace().into(),
            state: SimpleGoalState::Pending,
            on_done: on_done.map(|f| Box::new(f) as Box<_>),
            on_active: on_active.map(|f| Box::new(f) as Box<_>),
            on_feedback: on_feedback.map(|f| Box::new(f) as Box<_>),
        }));

        let handle_transition = {
            let callback_handle = Arc::clone(&callback_handle);
            move |gh| {
                callback_handle
                    .lock()
                    .expect(MUTEX_LOCK_FAIL)
                    .handle_transition(gh)
            }
        };

        let handle_feedback = {
            let callback_handle = Arc::clone(&callback_handle);
            move |gh, fb| {
                callback_handle
                    .lock()
                    .expect(MUTEX_LOCK_FAIL)
                    .handle_feedback(gh, fb)
            }
        };

        self.callback_handle = Some(callback_handle);

        let goal_handle =
            self.action_client
                .send_goal(goal, Some(handle_transition), Some(handle_feedback));

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

    pub fn wait_for_result(&self, _timeout: Option<Duration>) -> bool {
        unimplemented!()
    }

    pub fn result(&self) -> Option<ResultBody<T>> {
        let result = self.goal_handle.as_ref().and_then(|gh| gh.result());
        if result.is_none() {
            rosrust::ros_err!("Called result when no goal is running");
        }
        result
    }

    pub fn state(&self) -> GoalState {
        let inner_goal_state = self
            .goal_handle
            .as_ref()
            .map(|gh| gh.goal_state())
            .unwrap_or(GoalState::Lost);
        match inner_goal_state {
            GoalState::Recalling => GoalState::Pending,
            GoalState::Preempting => GoalState::Active,
            other => other,
        }
    }

    pub fn goal_status_text(&self) -> Option<String> {
        let status_text = self.goal_handle.as_ref().map(|gh| gh.goal_status_text());
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

#[allow(dead_code)]
struct CallbackStatus<T: Action> {
    expired: bool,
    namespace: String,
    state: SimpleGoalState,
    on_done: Option<Box<dyn Fn(GoalState, Option<ResultBody<T>>) + Send>>,
    on_active: Option<Box<dyn Fn() + Send>>,
    on_feedback: Option<Box<dyn Fn(FeedbackBody<T>) + Send>>,
}

impl<T: Action> CallbackStatus<T> {
    fn handle_transition(&mut self, gh: ClientGoalHandle<T>) {
        let comm_state = gh.comm_state();

        match (comm_state, self.state) {
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
                self.state = SimpleGoalState::Active;
                if let Some(ref on_active) = self.on_active {
                    (*on_active)();
                }
            }
            (State::Done, SimpleGoalState::Pending) | (State::Done, SimpleGoalState::Active) => {
                self.state = SimpleGoalState::Done;
                if let Some(ref on_done) = self.on_done {
                    (*on_done)(gh.goal_state(), gh.result());
                }
                // TODO: trigger done notification
            }
            _ => {}
        }
    }

    fn handle_feedback(&mut self, _gh: ClientGoalHandle<T>, feedback: FeedbackBody<T>) {
        if self.expired {
            return;
        }
        if let Some(ref on_feedback) = self.on_feedback {
            (*on_feedback)(feedback);
        }
    }
}

pub struct SendGoalBuilder<'a, T: Action, Fd, Fa, Ff> {
    client: &'a mut SimpleActionClient<T>,
    goal: GoalBody<T>,
    on_done: Option<Fd>,
    on_active: Option<Fa>,
    on_feedback: Option<Ff>,
}

impl<'a, T: Action, Fd, Fa, Ff> SendGoalBuilder<'a, T, Fd, Fa, Ff>
where
    Fd: Fn(GoalState, Option<ResultBody<T>>) + Send + 'static,
    Fa: Fn() + Send + 'static,
    Ff: Fn(FeedbackBody<T>) + Send + 'static,
{
    fn new(client: &'a mut SimpleActionClient<T>, goal: GoalBody<T>, _: Fd, _: Fa, _: Ff) -> Self {
        Self {
            client,
            goal,
            on_done: None,
            on_active: None,
            on_feedback: None,
        }
    }

    #[inline]
    pub fn on_done<Fnew>(self, callback: Fnew) -> SendGoalBuilder<'a, T, Fnew, Fa, Ff>
    where
        Fnew: Fn(GoalState, Option<ResultBody<T>>) + Send + 'static,
    {
        SendGoalBuilder {
            client: self.client,
            goal: self.goal,
            on_done: Some(callback),
            on_active: self.on_active,
            on_feedback: self.on_feedback,
        }
    }

    #[inline]
    pub fn on_active<Fnew>(self, callback: Fnew) -> SendGoalBuilder<'a, T, Fd, Fnew, Ff>
    where
        Fnew: Fn() + Send + 'static,
    {
        SendGoalBuilder {
            client: self.client,
            goal: self.goal,
            on_done: self.on_done,
            on_active: Some(callback),
            on_feedback: self.on_feedback,
        }
    }

    #[inline]
    pub fn on_feedback<Fnew>(self, callback: Fnew) -> SendGoalBuilder<'a, T, Fd, Fa, Fnew>
    where
        Fnew: Fn(FeedbackBody<T>) + Send + 'static,
    {
        SendGoalBuilder {
            client: self.client,
            goal: self.goal,
            on_done: self.on_done,
            on_active: self.on_active,
            on_feedback: Some(callback),
        }
    }

    #[inline]
    pub fn send(self) {
        self.client
            .send_goal(self.goal, self.on_done, self.on_active, self.on_feedback)
    }
}
