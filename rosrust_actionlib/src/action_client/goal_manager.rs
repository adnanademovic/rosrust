use super::comm_state_machine::{CommStateMachine, OnSendCancel, OnSendGoal};
use super::AsyncClientGoalHandle;
use crate::action_client::{OnFeedback, OnTransition};
use crate::static_messages::MUTEX_LOCK_FAIL;
use crate::GoalID;
use crate::{Action, FeedbackType, GoalBody, GoalType, ResultType};
use rosrust_msg::actionlib_msgs::GoalStatusArray;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex, Weak};

pub struct GoalManager<T: Action> {
    // TODO: This doesn't have to be a Vec. Consider alternatives.
    statuses: Vec<Weak<Mutex<CommStateMachine<T>>>>,
    on_send_goal: OnSendGoal<T>,
    on_cancel: OnSendCancel,
}

static NEXT_GOAL_ID: AtomicUsize = AtomicUsize::new(0);

fn generate_id() -> GoalID {
    let id = NEXT_GOAL_ID.fetch_add(1, Ordering::SeqCst);
    let stamp = rosrust::now();
    GoalID {
        id: format!("{}-{}-{}", rosrust::name(), id, stamp.seconds()),
        stamp,
    }
}

impl<T: Action> GoalManager<T> {
    pub fn new<Fsg, Fc>(on_send_goal: Fsg, on_cancel: Fc) -> Self
    where
        Fsg: Fn(T::Goal) + Send + Sync + 'static,
        Fc: Fn(GoalID) + Send + Sync + 'static,
    {
        Self {
            statuses: vec![],
            on_send_goal: Arc::new(on_send_goal),
            on_cancel: Arc::new(on_cancel),
        }
    }

    pub fn init_goal<'a>(
        &'a mut self,
        goal: GoalBody<T>,
        on_transition: Option<OnTransition<T>>,
        on_feedback: Option<OnFeedback<T>>,
    ) -> AsyncClientGoalHandle<T> {
        use crate::ActionGoal;

        let mut action_goal = GoalType::<T> {
            header: Default::default(),
            id: generate_id(),
            body: goal,
        };
        action_goal.header.stamp = rosrust::now();
        let init_action_goal = T::Goal::from_goal(action_goal.clone());

        let comm_state_machine = Arc::new(Mutex::new(CommStateMachine::new(
            action_goal,
            Arc::clone(&self.on_cancel),
        )));

        let mut csm_lock = comm_state_machine.lock().expect(MUTEX_LOCK_FAIL);

        if let Some(callback) = on_feedback {
            csm_lock.register_on_feedback(callback);
        }

        if let Some(callback) = on_transition {
            csm_lock.register_on_transition(callback);
        }

        drop(csm_lock);

        self.statuses.push(Arc::downgrade(&comm_state_machine));

        (*self.on_send_goal)(init_action_goal);

        AsyncClientGoalHandle::new(comm_state_machine)
    }

    fn for_each_status(&self, handler: impl Fn(&mut CommStateMachine<T>)) {
        for status in self.statuses.iter().filter_map(Weak::upgrade) {
            handler(&mut (status.lock().expect(MUTEX_LOCK_FAIL)))
        }
    }

    pub fn update_statuses(&mut self, status_array: &GoalStatusArray) {
        self.statuses.retain(|status| status.upgrade().is_some());
        self.for_each_status(|status| status.update_status(status_array));
    }

    pub fn update_results(&self, action_result: &ResultType<T>) {
        self.for_each_status(|status| status.update_result(action_result));
    }

    pub fn update_feedbacks(&self, action_feedback: &FeedbackType<T>) {
        self.for_each_status(|status| status.update_feedback(action_feedback));
    }
}
