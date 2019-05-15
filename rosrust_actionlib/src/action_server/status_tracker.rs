use super::goal_id_generator::GoalIdGenerator;
use crate::{Goal, GoalID, GoalState, GoalStatus};
use rosrust;
use std::sync::Arc;

pub struct StatusTracker<T> {
    pub goal: Option<Arc<Goal<T>>>,
    pub status: GoalStatus,
    pub destruction_time_set: bool,
    pub handle_destruction_time: rosrust::Time,
    pub id_generator: GoalIdGenerator,
}

impl<T: rosrust::Message> StatusTracker<T> {
    pub fn from_state(goal_id: GoalID, state: GoalState) -> Self {
        let goal = None;
        let id_generator = GoalIdGenerator::new();

        let status = GoalStatus {
            goal_id,
            state,
            text: String::new(),
        };

        let handle_destruction_time = rosrust::Time::default();
        Self {
            goal,
            status,
            destruction_time_set: false,
            handle_destruction_time,
            id_generator,
        }
    }

    pub fn from_goal(goal: Goal<T>) -> Self {
        let id_generator = GoalIdGenerator::new();

        let mut goal_id = if goal.id.id == "" {
            id_generator.generate_id()
        } else {
            goal.id.clone()
        };

        if goal_id.stamp.nanos() == 0 {
            goal_id.stamp = rosrust::now()
        }

        let status = GoalStatus {
            goal_id,
            state: GoalState::Pending,
            text: String::new(),
        };

        let handle_destruction_time = rosrust::Time::default();
        Self {
            goal: Some(Arc::new(goal)),
            status,
            destruction_time_set: false,
            handle_destruction_time,
            id_generator,
        }
    }

    pub fn refresh_destruction_time(&mut self) {
        if self.destruction_time_set {
            return;
        }
        self.destruction_time_set = true;
        self.handle_destruction_time = rosrust::now();
    }
}
