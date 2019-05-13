use crate::action_server::status_tracker::StatusTracker;
use crate::goal_status::{GoalID, GoalStatus};
use crate::static_messages::MUTEX_LOCK_FAIL;
use crate::{Action, ActionServerState, GoalBody, GoalType, ResultBody};
use rosrust::error::Result;
use std::sync::{Arc, Mutex};

pub struct ServerGoalHandle<T: Action> {
    goal: Option<Arc<GoalType<T>>>,
    _fields: Arc<Mutex<ActionServerState<T>>>,
    status_tracker: Arc<Mutex<StatusTracker<GoalBody<T>>>>,
}

// TODO: implement all missing methods
impl<T: Action> ServerGoalHandle<T> {
    pub(crate) fn new(
        fields: Arc<Mutex<ActionServerState<T>>>,
        status_tracker: Arc<Mutex<StatusTracker<GoalBody<T>>>>,
    ) -> Self {
        let goal = status_tracker.lock().expect(MUTEX_LOCK_FAIL).goal.clone();
        Self {
            goal,
            _fields: fields,
            status_tracker: status_tracker,
        }
    }

    pub fn default_result(&self) {
        unimplemented!()
    }

    pub fn set_accepted(&self) -> Result<()> {
        unimplemented!()
    }

    pub fn set_canceled(&self, _result: Option<ResultBody<T>>, _text: &str) -> Result<()> {
        unimplemented!()
    }

    pub fn set_rejected(&self) -> Result<()> {
        unimplemented!()
    }

    pub fn set_aborted(&self) -> Result<()> {
        unimplemented!()
    }

    pub fn set_succeeded(&self) -> Result<()> {
        unimplemented!()
    }

    pub fn publish_feedback(&self) -> Result<()> {
        unimplemented!()
    }

    pub fn goal(&self) -> Option<&GoalBody<T>> {
        self.goal.as_ref().map(|goal| &goal.body)
    }

    pub fn goal_id(&self) -> Option<GoalID> {
        if self.goal.is_none() {
            rosrust::ros_err!("Attempt to get a goal id on an uninitialized ServerGoalHandle");
            return None;
        }
        Some(
            self.status_tracker
                .lock()
                .expect(MUTEX_LOCK_FAIL)
                .status
                .goal_id
                .clone(),
        )
    }

    pub fn goal_status(&self) -> Option<GoalStatus> {
        Some(
            self.status_tracker
                .lock()
                .expect(MUTEX_LOCK_FAIL)
                .status
                .clone(),
        )
    }

    pub fn set_cancel_requested(&self) -> Result<bool> {
        unimplemented!();
    }
}
