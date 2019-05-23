use crate::{Goal, GoalID, GoalState, GoalStatus};
use rosrust::Time;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

pub struct StatusTracker<T> {
    goal: Option<Arc<Goal<T>>>,
    goal_id: GoalID,
    state: GoalState,
    text: String,
    destruction_time: Option<Time>,
}

impl<T: rosrust::Message> StatusTracker<T> {
    #[inline]
    pub fn new_cancelation(goal_id: GoalID, state: GoalState) -> Self {
        Self {
            goal: None,
            goal_id,
            state,
            text: String::new(),
            destruction_time: Some(rosrust::now()),
        }
    }

    pub fn new_goal(goal: Goal<T>) -> Self {
        let mut goal_id = if goal.id.id == "" {
            generate_id()
        } else {
            goal.id.clone()
        };

        if goal_id.stamp.nanos() == 0 {
            goal_id.stamp = rosrust::now();
        }

        Self {
            goal: Some(Arc::new(goal)),
            goal_id,
            state: GoalState::Pending,
            text: String::new(),
            destruction_time: None,
        }
    }

    #[inline]
    pub fn goal(&self) -> Option<Arc<Goal<T>>> {
        self.goal.clone()
    }

    #[inline]
    pub fn to_status(&self) -> GoalStatus {
        GoalStatus {
            goal_id: self.goal_id.clone(),
            state: self.state,
            text: self.text.clone(),
        }
    }

    #[inline]
    pub fn state(&self) -> GoalState {
        self.state
    }

    #[inline]
    pub fn goal_id(&self) -> &GoalID {
        &self.goal_id
    }

    #[inline]
    pub fn destruction_time(&self) -> Option<Time> {
        self.destruction_time
    }

    #[inline]
    pub fn set_state(&mut self, state: GoalState) {
        self.state = state;
    }

    #[inline]
    pub fn set_text(&mut self, text: &str) {
        self.text = text.into();
    }

    pub fn mark_for_destruction(&mut self, force: bool) {
        if !force && self.destruction_time.is_some() {
            return;
        }
        self.destruction_time = Some(rosrust::now());
    }
}

static GOAL_COUNT: AtomicUsize = AtomicUsize::new(1);

pub fn generate_id() -> GoalID {
    let seq_id = GOAL_COUNT.fetch_add(1, Ordering::SeqCst);
    let stamp = rosrust::now();
    let id = format!(
        "{name}-{seq_id}-{secs}.{nsecs}",
        name = rosrust::name(),
        seq_id = seq_id,
        secs = stamp.sec,
        nsecs = stamp.nsec,
    );

    GoalID { id, stamp }
}
