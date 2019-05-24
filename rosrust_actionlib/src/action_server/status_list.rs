use super::StatusTracker;
use crate::msg::actionlib_msgs::GoalStatusArray;
use crate::static_messages::MUTEX_LOCK_FAIL;
use crate::{Action, GoalBody};
use std::collections::BTreeMap;
use std::sync::{Arc, Mutex};

type Tracker<T> = Arc<Mutex<StatusTracker<GoalBody<T>>>>;

pub struct StatusList<T: Action> {
    timeout: i64,
    items: BTreeMap<String, Tracker<T>>,
    publisher: rosrust::Publisher<GoalStatusArray>,
}

impl<T: Action> StatusList<T> {
    pub fn new(timeout: i64, publisher: rosrust::Publisher<GoalStatusArray>) -> Self {
        Self {
            timeout,
            items: BTreeMap::new(),
            publisher,
        }
    }

    pub fn to_status_array(&mut self) -> GoalStatusArray {
        let now = rosrust::now();
        let now_nanos = now.nanos();
        let dead_keys = self
            .items
            .iter()
            .filter_map(|(key, tracker)| {
                let tracker = tracker.lock().expect(MUTEX_LOCK_FAIL);
                let destruction_time = tracker.destruction_time()?;
                if destruction_time.nanos() + self.timeout > now_nanos {
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
            self.items.remove(&key);
        }

        let status_list = self
            .items
            .values()
            .map(|tracker| tracker.lock().expect(MUTEX_LOCK_FAIL).to_status().into())
            .collect();
        GoalStatusArray {
            header: Default::default(),
            status_list,
        }
    }

    pub fn publish(&mut self) -> rosrust::error::Result<()> {
        let mut data = self.to_status_array();
        if !rosrust::is_ok() {
            return Ok(());
        }
        data.header.stamp = rosrust::now();
        self.publisher.send(data)
    }

    pub fn insert(&mut self, key: String, tracker: Tracker<T>) {
        self.items.insert(key, tracker);
    }

    pub fn get(&self, id: &str) -> Option<Tracker<T>> {
        self.items.get(id).cloned()
    }

    pub fn values(&self) -> impl Iterator<Item = &Tracker<T>> {
        self.items.values()
    }
}
