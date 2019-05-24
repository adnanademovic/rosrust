use super::{publish_response, ActionServerOnRequest, ServerGoalHandle, StatusTracker};
use crate::msg::actionlib_msgs::GoalStatusArray;
use crate::static_messages::MUTEX_LOCK_FAIL;
use crate::{Action, GoalBody, GoalID, GoalState, GoalType};
use rosrust::error::Result;
use std::collections::BTreeMap;
use std::sync::{Arc, Mutex, Weak};

pub(crate) struct ActionServerState<T: Action> {
    last_cancel_ns: i64,
    result_pub: rosrust::Publisher<T::Result>,
    feedback_pub: rosrust::Publisher<T::Feedback>,
    status_pub: rosrust::Publisher<GoalStatusArray>,
    status_frequency: f64,
    status_list: BTreeMap<String, Arc<Mutex<StatusTracker<GoalBody<T>>>>>,
    status_list_timeout: i64,
    on_goal: ActionServerOnRequest<T>,
    on_cancel: ActionServerOnRequest<T>,
    self_reference: Weak<Mutex<Self>>,
}

impl<T: Action> ActionServerState<T> {
    pub(crate) fn new(
        result_pub: rosrust::Publisher<T::Result>,
        feedback_pub: rosrust::Publisher<T::Feedback>,
        status_pub: rosrust::Publisher<GoalStatusArray>,
        status_frequency: f64,
        status_list_timeout: i64,
        on_goal: ActionServerOnRequest<T>,
        on_cancel: ActionServerOnRequest<T>,
    ) -> Arc<Mutex<Self>> {
        let output = Arc::new(Mutex::new(Self {
            last_cancel_ns: 0,
            result_pub,
            feedback_pub,
            status_pub,
            status_frequency,
            status_list: BTreeMap::new(),
            status_list_timeout,
            on_goal,
            on_cancel,
            self_reference: Weak::new(),
        }));

        output.lock().expect(MUTEX_LOCK_FAIL).self_reference = Arc::downgrade(&output);

        output
    }

    pub fn result_pub(&self) -> &rosrust::Publisher<T::Result> {
        &self.result_pub
    }

    pub fn feedback_pub(&self) -> &rosrust::Publisher<T::Feedback> {
        &self.feedback_pub
    }

    pub fn status_frequency(&self) -> f64 {
        self.status_frequency
    }

    fn get_status_array(&mut self) -> GoalStatusArray {
        let now = rosrust::now();
        let now_nanos = now.nanos();
        let status_list_timeout = self.status_list_timeout;
        let dead_keys = self
            .status_list
            .iter()
            .filter_map(|(key, tracker)| {
                let tracker = tracker.lock().expect(MUTEX_LOCK_FAIL);
                let destruction_time = tracker.destruction_time()?;
                if destruction_time.nanos() + status_list_timeout > now_nanos {
                    return None;
                }
                rosrust::ros_debug!(
                    "Item {} with destruction time of {} being removed from list.  Now = {}",
                    tracker.goal_id().id,
                    destruction_time.seconds(),
                    now.seconds()
                );
                Some(key)
            })
            .cloned()
            .collect::<Vec<_>>();
        for key in dead_keys {
            self.status_list.remove(&key);
        }

        let status_list = self
            .status_list
            .values()
            .map(|tracker| tracker.lock().expect(MUTEX_LOCK_FAIL).to_status().into())
            .collect();
        GoalStatusArray {
            header: Default::default(),
            status_list,
        }
    }

    pub fn publish_status(&mut self) -> Result<()> {
        let mut status_array = self.get_status_array();
        if !rosrust::is_ok() {
            return Ok(());
        }
        status_array.header.stamp = rosrust::now();
        self.status_pub.send(status_array)
    }

    pub fn handle_on_goal(&mut self, goal: GoalType<T>) -> Result<()> {
        rosrust::ros_debug!("The action server has received a new goal request");

        let goal_id = goal.id.id.clone();

        let existing_tracker = self.status_list.get(&goal_id);

        if let Some(tracker) = existing_tracker {
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

        let goal_handle = ServerGoalHandle::new(goal, &self, tracker)?;

        if goal_timestamp != 0 && goal_timestamp <= self.last_cancel_ns {
            goal_handle
                .response()
                .text("This goal handle was canceled by the action server because its timestamp is before the timestamp of the last cancel request")
                .send_canceled();
            return Ok(());
        };

        (*self.on_goal)(goal_handle)
    }

    pub fn self_reference(&self) -> Result<Arc<Mutex<Self>>> {
        self.self_reference
            .upgrade()
            .ok_or_else(|| "Action Server was deconstructed before action was handled".into())
    }

    pub fn handle_on_cancel(&mut self, goal_id: GoalID) -> Result<()> {
        rosrust::ros_debug!("The action server has received a new cancel request");

        let filter_id = &goal_id.id;
        let filter_stamp = goal_id.stamp.nanos();

        let cancel_everything = filter_id == "" && filter_stamp == 0;

        let mut goal_id_found = false;

        for tracker in self.status_list.values() {
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
                let goal_handle = ServerGoalHandle::new(goal, &self, tracker_ref)?;

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

        if filter_stamp > self.last_cancel_ns {
            self.last_cancel_ns = filter_stamp;
        }
        Ok(())
    }

    #[inline]
    pub fn set_on_goal(&mut self, on_goal: ActionServerOnRequest<T>) {
        self.on_goal = on_goal;
    }

    #[inline]
    pub fn set_on_cancel(&mut self, on_cancel: ActionServerOnRequest<T>) {
        self.on_cancel = on_cancel;
    }

    fn add_status_tracker(&mut self, key: String, tracker: Arc<Mutex<StatusTracker<GoalBody<T>>>>) {
        self.status_list.insert(key, tracker);
    }
}
