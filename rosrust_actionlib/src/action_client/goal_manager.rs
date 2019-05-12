use crate::action_client::comm_state_machine::CommStateMachine;
use crate::action_client::ClientGoalHandle;
use crate::goal_status::GoalID;
use crate::static_messages::MUTEX_LOCK_FAIL;
use crate::{Action, FeedbackBody, GoalBody, GoalType};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex, Weak};

type OnSendGoal<T> = Arc<Fn(ClientGoalHandle<T>) + Send + Sync + 'static>;
type OnCancel<T> = Arc<Fn(ClientGoalHandle<T>, GoalID) + Send + Sync + 'static>;

pub struct GoalManager<T: Action> {
    statuses: Mutex<Vec<Weak<Mutex<CommStateMachine<T>>>>>,
    on_send_goal: OnSendGoal<T>,
    on_cancel: OnCancel<T>,
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
        Fsg: Fn(ClientGoalHandle<T>) + Send + Sync + 'static,
        Fc: Fn(ClientGoalHandle<T>, GoalID) + Send + Sync + 'static,
    {
        Self {
            statuses: Mutex::new(vec![]),
            on_send_goal: Arc::new(on_send_goal),
            on_cancel: Arc::new(on_cancel),
        }
    }

    // TODO: make more ergonomic, probably with a builder
    pub fn init_goal(
        &self,
        goal: GoalBody<T>,
        on_transition: Option<Box<dyn Fn() + Send + Sync + 'static>>,
        on_feedback: Option<Box<dyn Fn(FeedbackBody<T>) + Send + Sync + 'static>>,
    ) -> ClientGoalHandle<T> {
        let mut action_goal = GoalType::<T> {
            header: Default::default(),
            id: generate_id(),
            body: goal,
        };
        action_goal.header.stamp = rosrust::now();

        let comm_state_machine = Arc::new(Mutex::new(CommStateMachine::new(
            action_goal,
            on_feedback,
            on_transition,
        )));

        let mut csm_lock = comm_state_machine.lock().expect(MUTEX_LOCK_FAIL);

        csm_lock.register_send_goal({
            let callback = Arc::clone(&self.on_send_goal);
            let csm = Arc::downgrade(&comm_state_machine);
            move || {
                if let Some(csm) = Weak::upgrade(&csm) {
                    callback(ClientGoalHandle::new(csm))
                }
            }
        });

        csm_lock.register_send_cancel({
            let callback = Arc::clone(&self.on_cancel);
            let csm = Arc::downgrade(&comm_state_machine);
            move |goal_id| {
                if let Some(csm) = Weak::upgrade(&csm) {
                    callback(ClientGoalHandle::new(csm), goal_id)
                }
            }
        });

        drop(csm_lock);

        self.statuses
            .lock()
            .expect(MUTEX_LOCK_FAIL)
            .push(Arc::downgrade(&comm_state_machine));

        return ClientGoalHandle::new(comm_state_machine);
    }
}
