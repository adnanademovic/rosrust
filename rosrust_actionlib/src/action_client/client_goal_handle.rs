use crate::action_client::comm_state_machine::{CommStateMachine, State};
use crate::static_messages::MUTEX_LOCK_FAIL;
use crate::{Action, ResultBody};
use crate::{GoalID, GoalState};
use std::sync::{Arc, Mutex};

pub trait ClientGoalHandle<T: Action> {
    fn cancel(&self);
    fn comm_state(&self) -> State;
    fn goal_id(&self) -> GoalID;
    fn goal_state(&self) -> GoalState;
    fn goal_status_text(&self) -> String;
    fn result(&self) -> Option<ResultBody<T>>;
    fn get_terminal_state(&self) -> GoalState;
}

pub struct AsyncClientGoalHandle<T: Action> {
    state_machine: Arc<Mutex<CommStateMachine<T>>>,
}

pub struct SyncClientGoalHandle<'a, T: Action> {
    state_machine: &'a CommStateMachine<T>,
}

impl<T: Action> AsyncClientGoalHandle<T> {
    pub(crate) fn new(state_machine: Arc<Mutex<CommStateMachine<T>>>) -> Self {
        Self { state_machine }
    }
}

impl<T: Action> ClientGoalHandle<T> for AsyncClientGoalHandle<T> {
    fn cancel(&self) {
        let state_machine = self.state_machine.lock().expect(MUTEX_LOCK_FAIL);
        SyncClientGoalHandle::new(&state_machine).cancel()
    }

    #[inline]
    fn comm_state(&self) -> State {
        let state_machine = self.state_machine.lock().expect(MUTEX_LOCK_FAIL);
        SyncClientGoalHandle::new(&state_machine).comm_state()
    }

    #[inline]
    fn goal_id(&self) -> GoalID {
        let state_machine = self.state_machine.lock().expect(MUTEX_LOCK_FAIL);
        SyncClientGoalHandle::new(&state_machine).goal_id()
    }

    #[inline]
    fn goal_state(&self) -> GoalState {
        let state_machine = self.state_machine.lock().expect(MUTEX_LOCK_FAIL);
        SyncClientGoalHandle::new(&state_machine).goal_state()
    }

    #[inline]
    fn goal_status_text(&self) -> String {
        let state_machine = self.state_machine.lock().expect(MUTEX_LOCK_FAIL);
        SyncClientGoalHandle::new(&state_machine).goal_status_text()
    }

    #[inline]
    fn result(&self) -> Option<ResultBody<T>> {
        let state_machine = self.state_machine.lock().expect(MUTEX_LOCK_FAIL);
        SyncClientGoalHandle::new(&state_machine).result()
    }

    #[inline]
    fn get_terminal_state(&self) -> GoalState {
        let state_machine = self.state_machine.lock().expect(MUTEX_LOCK_FAIL);
        SyncClientGoalHandle::new(&state_machine).get_terminal_state()
    }
}

impl<'a, T: Action> SyncClientGoalHandle<'a, T> {
    #[inline]
    pub(crate) fn new(state_machine: &'a CommStateMachine<T>) -> Self {
        Self { state_machine }
    }
}

impl<'a, T: Action> ClientGoalHandle<T> for SyncClientGoalHandle<'a, T> {
    fn cancel(&self) {
        let cancel_message = GoalID {
            stamp: rosrust::Time::new(),
            id: self.state_machine.action_goal().id.id.clone(),
        };
        self.state_machine.send_cancel(cancel_message);
        self.state_machine.transition_to(State::WaitingForCancelAck)
    }

    #[inline]
    fn comm_state(&self) -> State {
        self.state_machine.state()
    }

    #[inline]
    fn goal_id(&self) -> GoalID {
        self.state_machine.action_goal().id.clone()
    }

    #[inline]
    fn goal_state(&self) -> GoalState {
        self.state_machine.latest_goal_status().state
    }

    #[inline]
    fn goal_status_text(&self) -> String {
        self.state_machine.latest_goal_status().text.clone()
    }

    #[inline]
    fn result(&self) -> Option<ResultBody<T>> {
        self.state_machine
            .latest_result()
            .as_ref()
            .map(|result| result.body.clone())
    }

    fn get_terminal_state(&self) -> GoalState {
        if self.state_machine.state() != State::Done {
            rosrust::ros_warn!(
                "Asking for the terminal state when we're in [{:?}]",
                self.state_machine.state(),
            );
        }
        let goal_state = self.state_machine.latest_goal_status().state;
        match goal_state {
            GoalState::Preempted
            | GoalState::Succeeded
            | GoalState::Aborted
            | GoalState::Rejected
            | GoalState::Recalled
            | GoalState::Lost => goal_state,
            GoalState::Pending
            | GoalState::Active
            | GoalState::Preempting
            | GoalState::Recalling => {
                rosrust::ros_err!(
                    "Asking for a terminal state, but the goal status is {:?}",
                    goal_state
                );
                GoalState::Lost
            }
        }
    }
}
