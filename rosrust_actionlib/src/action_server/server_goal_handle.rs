use crate::action_server::status_tracker::StatusTracker;
use crate::goal_status::{GoalID, GoalState, GoalStatus};
use crate::static_messages::MUTEX_LOCK_FAIL;
use crate::{Action, ActionServerState, GoalBody, GoalType, ResultBody};
use rosrust::error::Result;
use std::convert::TryInto;
use std::sync::{Arc, Mutex};

pub struct ServerGoalHandle<T: Action> {
    goal: Option<Arc<GoalType<T>>>,
    action_server: Arc<Mutex<ActionServerState<T>>>,
    status_tracker: Arc<Mutex<StatusTracker<GoalBody<T>>>>,
}

// TODO: implement all missing methods
impl<T: Action> ServerGoalHandle<T> {
    pub(crate) fn new(
        action_server: Arc<Mutex<ActionServerState<T>>>,
        status_tracker: Arc<Mutex<StatusTracker<GoalBody<T>>>>,
    ) -> Self {
        let goal = status_tracker.lock().expect(MUTEX_LOCK_FAIL).goal.clone();
        Self {
            goal,
            action_server,
            status_tracker: status_tracker,
        }
    }

    pub fn create_default_result(&self) -> ResultBody<T> {
        Default::default()
    }

    pub fn set_accepted(&self, text: &str) {
        let goal_id = self.goal_id();
        rosrust::ros_debug!(
            "Accepting goal, id: {}, stamp: {}",
            goal_id.id,
            goal_id.stamp.seconds()
        );
        if self.goal.is_none() {
            rosrust::ros_err!("Attempt to set status on an uninitialized ServerGoalHandle");
            return;
        }
        let mut status_tracker = self.status_tracker.lock().expect(MUTEX_LOCK_FAIL);
        match status_tracker
            .status
            .status
            .try_into()
            .unwrap_or(GoalState::Lost)
        {
            GoalState::Pending => {
                status_tracker.status.status = GoalState::Active as u8;
            }
            GoalState::Recalling => {
                status_tracker.status.status = GoalState::Preempting as u8;
            }
            status => {
                rosrust::ros_err!("To transition to an active state, the goal must be in a pending or recalling state, it is currently in state: {:?}",  status);
                return;
            }
        }
        status_tracker.status.text = text.into();
        if let Err(err) = self
            .action_server
            .lock()
            .expect(MUTEX_LOCK_FAIL)
            .publish_status()
        {
            rosrust::ros_err!("Failed to publish status: {}", err);
        }
    }

    pub fn set_canceled(&self, _result: Option<ResultBody<T>>, _text: &str) {
        unimplemented!()
    }

    pub fn set_rejected(&self) {
        unimplemented!()
    }

    pub fn set_aborted(&self) {
        unimplemented!()
    }

    pub fn set_succeeded(&self) {
        unimplemented!()
    }

    pub fn publish_feedback(&self) {
        unimplemented!()
    }

    pub fn goal(&self) -> Option<&GoalBody<T>> {
        Some(&self.goal.as_ref()?.body)
    }

    pub fn goal_id(&self) -> GoalID {
        if self.goal.is_none() {
            rosrust::ros_err!("Attempt to get a goal id on an uninitialized ServerGoalHandle");
            return GoalID::default();
        }

        self.status_tracker
            .lock()
            .expect(MUTEX_LOCK_FAIL)
            .status
            .goal_id
            .clone()
    }

    pub fn goal_status(&self) -> GoalStatus {
        if self.goal.is_none() {
            rosrust::ros_err!("Attempt to get a goal status on an uninitialized ServerGoalHandle");
            return GoalStatus::default();
        }

        self.status_tracker
            .lock()
            .expect(MUTEX_LOCK_FAIL)
            .status
            .clone()
    }

    pub fn set_cancel_requested(&self) -> Result<bool> {
        unimplemented!();
    }
}
