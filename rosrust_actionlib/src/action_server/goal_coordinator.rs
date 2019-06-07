use super::{publish_response, ActionServerOnRequest, ServerGoalHandle, StatusList, StatusTracker};
use crate::static_messages::MUTEX_LOCK_FAIL;
use crate::{Action, GoalBody, GoalID, GoalState, GoalType};
use rosrust::error::Result;
use std::sync::{Arc, Mutex};

pub struct GoalCoordinator<T: Action> {
    last_cancel_ns: Arc<Mutex<i64>>,
    result_pub: rosrust::Publisher<T::Result>,
    feedback_pub: rosrust::Publisher<T::Feedback>,
    status_list: Arc<Mutex<StatusList<T>>>,
    on_goal: ActionServerOnRequest<T>,
    on_cancel: ActionServerOnRequest<T>,
}

impl<T: Action> GoalCoordinator<T> {
    pub fn new(
        result_pub: rosrust::Publisher<T::Result>,
        feedback_pub: rosrust::Publisher<T::Feedback>,
        on_goal: ActionServerOnRequest<T>,
        on_cancel: ActionServerOnRequest<T>,
        status_list: Arc<Mutex<StatusList<T>>>,
    ) -> Self {
        Self {
            last_cancel_ns: Arc::new(Mutex::new(0)),
            result_pub,
            feedback_pub,
            status_list,
            on_goal,
            on_cancel,
        }
    }

    pub fn handle_on_goal(&self, goal: GoalType<T>) -> Result<()> {
        rosrust::ros_debug!("The action server has received a new goal request");

        let tracker_option = self
            .status_list
            .lock()
            .expect(MUTEX_LOCK_FAIL)
            .get(&goal.id.id);

        if let Some(tracker) = tracker_option {
            self.handle_on_goal_existing(goal, tracker)
        } else {
            self.handle_on_goal_new(goal)
        }
    }

    fn handle_on_goal_existing(
        &self,
        goal: GoalType<T>,
        tracker: Arc<Mutex<StatusTracker<GoalBody<T>>>>,
    ) -> Result<()> {
        let mut tracker = tracker.lock().expect(MUTEX_LOCK_FAIL);

        rosrust::ros_debug!(
            "Goal {} was already in the status list with status {:?}",
            goal.id.id,
            tracker.state()
        );

        tracker.mark_for_destruction(false);

        if tracker.state() == GoalState::Recalling {
            tracker.set_state(GoalState::Recalled);
            let status = tracker.to_status();
            drop(tracker);

            publish_response(&self.result_pub, status, Default::default())?;
        }

        Ok(())
    }

    fn handle_on_goal_new(&self, goal: GoalType<T>) -> Result<()> {
        let tracker = StatusTracker::new_goal(goal);
        let goal_timestamp = tracker.goal_id().stamp.nanos();

        let key = tracker.goal_id().id.clone();
        // This is guaranteed to return Some because it's not a cancellation status tracker
        let goal = tracker.goal().unwrap();
        let tracker = Arc::new(Mutex::new(tracker));
        self.add_status_tracker(key, Arc::clone(&tracker));

        let goal_handle = ServerGoalHandle::new(
            self.result_pub.clone(),
            self.feedback_pub.clone(),
            goal,
            self.status_list.clone(),
            tracker,
        )?;

        if goal_timestamp != 0 && goal_timestamp <= self.last_cancel() {
            goal_handle
                .response()
                .text("This goal handle was canceled by the action server because its timestamp is before the timestamp of the last cancel request")
                .send_canceled();
            Ok(())
        } else {
            (*self.on_goal)(goal_handle)
        }
    }

    pub fn handle_on_cancel(&self, goal_id: GoalID) -> Result<()> {
        rosrust::ros_debug!("The action server has received a new cancel request");

        let goal_filter: GoalFilter = goal_id.clone().into();

        let mut goal_id_found = goal_filter.id().is_none();

        let canceled_trackers: Vec<Arc<_>> = {
            let status_list = self.status_list.lock().expect(MUTEX_LOCK_FAIL);
            if goal_filter.matches_everything() {
                status_list.values().cloned().collect::<Vec<_>>()
            } else {
                status_list
                    .values()
                    .filter(|tracker| {
                        let tracker = tracker.lock().expect(MUTEX_LOCK_FAIL);
                        goal_filter.matches(&tracker.goal_id())
                    })
                    .cloned()
                    .collect::<Vec<_>>()
            }
        };

        for tracker_ref in canceled_trackers {
            let mut tracker = tracker_ref.lock().expect(MUTEX_LOCK_FAIL);
            if goal_filter.matches_id(&tracker.goal_id().id) {
                goal_id_found = true;
            }
            tracker.mark_for_destruction(false);
            let goal = tracker.goal();
            drop(tracker);

            if let Some(goal) = goal {
                let goal_handle = ServerGoalHandle::new(
                    self.result_pub.clone(),
                    self.feedback_pub.clone(),
                    goal,
                    self.status_list.clone(),
                    tracker_ref,
                )?;

                if goal_handle.set_cancel_requested() {
                    (*self.on_cancel)(goal_handle)?;
                }
            }
        }

        if !goal_id_found {
            let tracker = StatusTracker::new_cancellation(goal_id, GoalState::Recalling);
            let key = tracker.goal_id().id.clone();
            self.add_status_tracker(key, Arc::new(Mutex::new(tracker)));
        }

        if let Some(ref stamp) = goal_filter.stamp() {
            let mut last_cancel_ns = self.last_cancel_ns.lock().expect(MUTEX_LOCK_FAIL);
            if *stamp > *last_cancel_ns {
                *last_cancel_ns = *stamp;
            }
        }

        Ok(())
    }

    fn add_status_tracker(&self, key: String, tracker: Arc<Mutex<StatusTracker<GoalBody<T>>>>) {
        self.status_list
            .lock()
            .expect(MUTEX_LOCK_FAIL)
            .insert(key, tracker);
    }

    fn last_cancel(&self) -> i64 {
        *self.last_cancel_ns.lock().expect(MUTEX_LOCK_FAIL)
    }
}

struct GoalFilter {
    id: Option<String>,
    stamp_nanos: Option<i64>,
}

impl From<GoalID> for GoalFilter {
    fn from(goal_id: GoalID) -> Self {
        let id = if goal_id.id == "" {
            None
        } else {
            Some(goal_id.id)
        };
        let nanos = goal_id.stamp.nanos();
        let stamp_nanos = if nanos == 0 { None } else { Some(nanos) };
        Self { id, stamp_nanos }
    }
}

impl GoalFilter {
    fn matches_everything(&self) -> bool {
        self.id.is_none() && self.stamp_nanos.is_none()
    }

    fn matches(&self, goal_id: &GoalID) -> bool {
        self.matches_id(&goal_id.id) || self.matches_stamp(goal_id.stamp)
    }

    fn matches_id(&self, other_id: &str) -> bool {
        self.id.as_ref().map(|id| id == other_id).unwrap_or(false)
    }

    fn id(&self) -> &Option<String> {
        &self.id
    }

    fn matches_stamp(&self, other_stamp: rosrust::Time) -> bool {
        self.stamp_nanos
            .map(|stamp| other_stamp.nanos() <= stamp)
            .unwrap_or(false)
    }

    fn stamp(&self) -> &Option<i64> {
        &self.stamp_nanos
    }
}
