use crate::action_client::comm_state_machine::{CommStateMachine, State};
use crate::goal_status::{GoalID, GoalState};
use crate::static_messages::MUTEX_LOCK_FAIL;
use crate::{Action, ResultBody};
use std::convert::TryInto;
use std::sync::{Arc, Mutex};

pub struct ClientGoalHandle<T: Action> {
    state_machine: Arc<Mutex<CommStateMachine<T>>>,
}

impl<T: Action> ClientGoalHandle<T> {
    pub(crate) fn new(state_machine: Arc<Mutex<CommStateMachine<T>>>) -> Self {
        Self { state_machine }
    }

    pub fn cancel(&self) {
        let mut state_machine = self.state_machine.lock().expect(MUTEX_LOCK_FAIL);
        let cancel_message = GoalID {
            stamp: rosrust::Time::new(),
            id: state_machine.action_goal().id.id.clone(),
        };
        state_machine.send_cancel(cancel_message);
        state_machine.transition_to(State::WaitingForCancelAck)
    }

    #[inline]
    pub fn comm_state(&self) -> State {
        self.state_machine.lock().expect(MUTEX_LOCK_FAIL).state()
    }

    #[inline]
    pub fn goal_status(&self) -> GoalState {
        self.state_machine
            .lock()
            .expect(MUTEX_LOCK_FAIL)
            .latest_goal_status()
            .status
            .try_into()
            .unwrap_or(GoalState::Lost)
    }

    #[inline]
    pub fn goal_status_text(&self) -> String {
        self.state_machine
            .lock()
            .expect(MUTEX_LOCK_FAIL)
            .latest_goal_status()
            .text
            .clone()
    }

    #[inline]
    pub fn result(&self) -> Option<ResultBody<T>> {
        self.state_machine
            .lock()
            .expect(MUTEX_LOCK_FAIL)
            .latest_result()
            .as_ref()
            .map(|result| result.body.clone())
    }

    #[inline]
    pub fn get_terminal_state(&self) -> GoalState {
        let state_machine = self.state_machine.lock().expect(MUTEX_LOCK_FAIL);
        if state_machine.state() != State::Done {
            rosrust::ros_warn!(
                "Asking for the terminal state when we're in [{:?}]",
                state_machine.state(),
            );
        }
        let goal_state: GoalState = state_machine
            .latest_goal_status()
            .status
            .try_into()
            .unwrap_or(GoalState::Lost);
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
