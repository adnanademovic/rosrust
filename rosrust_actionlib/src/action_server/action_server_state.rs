use super::{publish_response, ActionServerOnRequest, ServerGoalHandle, StatusList, StatusTracker};
use crate::static_messages::MUTEX_LOCK_FAIL;
use crate::{Action, GoalBody, GoalID, GoalState, GoalType};
use rosrust::error::Result;
use std::sync::{Arc, Mutex};

pub(crate) struct ActionServerState<T: Action> {
    last_cancel_ns: Arc<Mutex<i64>>,
    result_pub: rosrust::Publisher<T::Result>,
    feedback_pub: rosrust::Publisher<T::Feedback>,
    status_list: Arc<Mutex<StatusList<T>>>,
    on_goal: ActionServerOnRequest<T>,
    on_cancel: ActionServerOnRequest<T>,
}

impl<T: Action> ActionServerState<T> {
    pub(crate) fn new(
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

        let goal_id = goal.id.id.clone();

        if let Some(tracker) = self
            .status_list
            .lock()
            .expect(MUTEX_LOCK_FAIL)
            .get(&goal_id)
        {
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

                publish_response(&self.result_pub, status, Default::default())?;
            }

            return Ok(());
        }

        let tracker = StatusTracker::new_goal(goal);
        let goal_timestamp = tracker.goal_id().stamp.nanos();

        let key = tracker.goal_id().id.clone();
        // This is guaranteed to return Some because it's not a cancelation status tracker
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

        if goal_timestamp != 0
            && goal_timestamp <= *self.last_cancel_ns.lock().expect(MUTEX_LOCK_FAIL)
        {
            goal_handle
                .response()
                .text("This goal handle was canceled by the action server because its timestamp is before the timestamp of the last cancel request")
                .send_canceled();
            return Ok(());
        };

        (*self.on_goal)(goal_handle)
    }

    pub fn handle_on_cancel(&self, goal_id: GoalID) -> Result<()> {
        rosrust::ros_debug!("The action server has received a new cancel request");

        let filter_id = &goal_id.id;
        let filter_stamp = goal_id.stamp.nanos();

        let cancel_everything = filter_id == "" && filter_stamp == 0;

        let mut goal_id_found = false;

        for tracker in self.status_list.lock().expect(MUTEX_LOCK_FAIL).values() {
            let tracker_ref = Arc::clone(&tracker);
            let mut tracker = tracker.lock().expect(MUTEX_LOCK_FAIL);
            let cancel_this = filter_id == &tracker.goal_id().id;
            let cancel_before_stamp =
                filter_stamp != 0 && tracker.goal_id().stamp.nanos() <= filter_stamp;
            if !cancel_everything && !cancel_this && !cancel_before_stamp {
                continue;
            }
            goal_id_found = goal_id_found || filter_id == &tracker.goal_id().id;
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

        if filter_id != "" && !goal_id_found {
            let tracker = StatusTracker::new_cancelation(goal_id, GoalState::Recalling);
            let key = tracker.goal_id().id.clone();
            self.add_status_tracker(key, Arc::new(Mutex::new(tracker)));
        }

        let mut last_cancel_ns = self.last_cancel_ns.lock().expect(MUTEX_LOCK_FAIL);
        if filter_stamp > *last_cancel_ns {
            *last_cancel_ns = filter_stamp;
        }
        Ok(())
    }

    fn add_status_tracker(&self, key: String, tracker: Arc<Mutex<StatusTracker<GoalBody<T>>>>) {
        self.status_list
            .lock()
            .expect(MUTEX_LOCK_FAIL)
            .insert(key, tracker);
    }
}
