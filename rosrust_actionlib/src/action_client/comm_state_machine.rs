use crate::action_client::{OnFeedback, OnTransition, SyncClientGoalHandle};
use crate::{Action, FeedbackType, GoalType, ResultType};
use crate::{GoalID, GoalState, GoalStatus};
use rosrust_msg::actionlib_msgs::GoalStatusArray;
use std::cell::Cell;
use std::sync::Arc;

pub type OnSendGoal<T> = Arc<dyn Fn(<T as Action>::Goal) + Send + Sync + 'static>;
pub type OnSendCancel = Arc<dyn Fn(GoalID) + Send + Sync + 'static>;

pub struct CommStateMachine<T: Action> {
    action_goal: GoalType<T>,
    on_feedback: Option<OnFeedback<T>>,
    on_transition: Option<OnTransition<T>>,
    send_cancel_handler: OnSendCancel,
    state: Cell<State>,
    latest_goal_status: GoalStatus,
    latest_result: Option<ResultType<T>>,
}

impl<T: Action> CommStateMachine<T> {
    pub(crate) fn new(action_goal: GoalType<T>, send_cancel_handler: OnSendCancel) -> Self {
        Self {
            action_goal,
            send_cancel_handler,
            state: Cell::new(State::WaitingForGoalAck),
            latest_goal_status: GoalStatus::default(),
            on_feedback: None,
            on_transition: None,
            latest_result: None,
        }
    }

    pub(crate) fn register_on_feedback(&mut self, f: OnFeedback<T>) {
        self.on_feedback = Some(f);
    }

    pub(crate) fn register_on_transition(&mut self, f: OnTransition<T>) {
        self.on_transition = Some(f);
    }

    #[inline]
    pub fn send_cancel(&self, msg: GoalID) {
        (*self.send_cancel_handler)(msg);
    }

    #[inline]
    pub fn action_goal(&self) -> &GoalType<T> {
        &self.action_goal
    }

    #[inline]
    pub fn state(&self) -> State {
        self.state.get()
    }

    #[inline]
    pub fn latest_goal_status(&self) -> &GoalStatus {
        &self.latest_goal_status
    }

    #[inline]
    pub fn latest_result(&self) -> &Option<ResultType<T>> {
        &self.latest_result
    }

    pub fn transition_to(&self, state: State) {
        let old_state = self.state.replace(state);
        rosrust::ros_debug!(
            "Transitioning to {:?} (from {:?}, goal: {})",
            state,
            old_state,
            self.action_goal.id.id,
        );

        if let Some(on_transition) = &self.on_transition {
            on_transition(SyncClientGoalHandle::new(self))
        }
    }

    pub fn update_feedback(&mut self, action_feedback: &FeedbackType<T>) {
        if self.action_goal.id.id != action_feedback.status.goal_id.id {
            return;
        }

        if self.state() == State::Done {
            return;
        }

        if let Some(on_feedback) = &self.on_feedback {
            on_feedback(
                SyncClientGoalHandle::new(self),
                action_feedback.body.clone(),
            )
        }
    }

    pub fn update_result(&mut self, action_result: &ResultType<T>) {
        if self.action_goal.id.id != action_result.status.goal_id.id {
            return;
        }

        let status = action_result.status.clone();

        self.latest_goal_status = status.clone();
        self.latest_result = Some(action_result.clone());

        match self.state() {
            State::WaitingForGoalAck
            | State::Pending
            | State::Active
            | State::WaitingForResult
            | State::WaitingForCancelAck
            | State::Recalling
            | State::Preempting => {
                let mut status_array = GoalStatusArray::default();
                status_array.status_list.push(status.into());
                self.update_status(&status_array);
                self.transition_to(State::Done);
            }
            State::Done => {
                rosrust::ros_err!("Got a result when we were already in the DONE state");
            }
            State::Lost => {
                rosrust::ros_err!("In a funny state: Lost");
            }
        };
    }

    fn update_status_inner(
        &mut self,
        status_array: &GoalStatusArray,
    ) -> Result<(), Option<String>> {
        use std::convert::TryInto;

        if self.state() == State::Done {
            return Err(None);
        }
        let status = status_array
            .status_list
            .iter()
            .find(|status| status.goal_id.id == self.action_goal.id.id)
            .cloned()
            .ok_or_else(|| {
                match self.state() {
                    State::Pending
                    | State::Active
                    | State::WaitingForCancelAck
                    | State::Recalling
                    | State::Preempting
                    | State::Lost => self.mark_as_lost(),
                    State::WaitingForGoalAck | State::WaitingForResult | State::Done => {}
                }
                None
            })?;

        let goal_state = status
            .status
            .try_into()
            .map_err(|err| format!("Got an unknown status from the ActionServer: {}", err))?;

        self.latest_goal_status = status.into();

        let steps = self
            .state
            .get()
            .transition_to(goal_state)
            .into_update_status_steps()?;

        for step in steps {
            self.transition_to(*step);
        }

        Ok(())
    }

    pub fn update_status(&mut self, status_array: &GoalStatusArray) {
        if let Err(Some(err)) = self.update_status_inner(status_array) {
            rosrust::ros_err!("{}", err);
        }
    }

    fn mark_as_lost(&mut self) {
        self.latest_goal_status.state = GoalState::Lost;
        self.transition_to(State::Done);
    }
}

// TODO: consider removing "Lost"
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum State {
    WaitingForGoalAck,
    Pending,
    Active,
    WaitingForResult,
    WaitingForCancelAck,
    Recalling,
    Preempting,
    Done,
    Lost,
}

#[derive(Clone, Debug)]
pub enum Transition {
    None,
    Invalid(State, GoalState),
    FunnyState(State),
    UnknownStatus(GoalState),
    Steps(&'static [State]),
}

impl Transition {
    fn into_update_status_steps(self) -> Result<&'static [State], String> {
        match self {
            Transition::None => Ok(&[]),
            Transition::Invalid(state, goal_state) => Err(format!(
                "Invalid goal status transition from {:?} to {:?}",
                state, goal_state,
            )),
            Transition::FunnyState(state) => {
                Err(format!("CommStateMachine is in a funny state: {:?}", state))
            }
            Transition::UnknownStatus(goal_state) => Err(format!(
                "Got an unknown status from the ActionServer: {:?}",
                goal_state,
            )),
            Transition::Steps(steps) => Ok(steps),
        }
    }
}

impl State {
    pub fn transition_to(self, goal: GoalState) -> Transition {
        match self {
            State::WaitingForGoalAck => match goal {
                GoalState::Pending => Transition::Steps(&[State::Pending]),
                GoalState::Active => Transition::Steps(&[State::Active]),
                GoalState::Rejected => {
                    Transition::Steps(&[State::Pending, State::WaitingForResult])
                }
                GoalState::Recalling => Transition::Steps(&[State::Pending, State::Recalling]),
                GoalState::Recalled => {
                    Transition::Steps(&[State::Pending, State::WaitingForResult])
                }
                GoalState::Preempted => {
                    Transition::Steps(&[State::Active, State::Preempting, State::WaitingForResult])
                }
                GoalState::Succeeded => {
                    Transition::Steps(&[State::Active, State::WaitingForResult])
                }
                GoalState::Aborted => Transition::Steps(&[State::Active, State::WaitingForResult]),
                GoalState::Preempting => Transition::Steps(&[State::Active, State::Preempting]),
                GoalState::Lost => Transition::UnknownStatus(GoalState::Lost),
            },
            State::Pending => match goal {
                GoalState::Pending => Transition::None,
                GoalState::Active => Transition::Steps(&[State::Active]),
                GoalState::Rejected => Transition::Steps(&[State::WaitingForResult]),
                GoalState::Recalling => Transition::Steps(&[State::Recalling]),
                GoalState::Recalled => {
                    Transition::Steps(&[State::Recalling, State::WaitingForResult])
                }
                GoalState::Preempted => {
                    Transition::Steps(&[State::Active, State::Preempting, State::WaitingForResult])
                }
                GoalState::Succeeded => {
                    Transition::Steps(&[State::Active, State::WaitingForResult])
                }
                GoalState::Aborted => Transition::Steps(&[State::Active, State::WaitingForResult]),
                GoalState::Preempting => Transition::Steps(&[State::Active, State::Preempting]),
                GoalState::Lost => Transition::UnknownStatus(GoalState::Lost),
            },
            State::Active => match goal {
                GoalState::Pending => Transition::Invalid(self, goal),
                GoalState::Active => Transition::None,
                GoalState::Rejected => Transition::Invalid(self, goal),
                GoalState::Recalling => Transition::Invalid(self, goal),
                GoalState::Recalled => Transition::Invalid(self, goal),
                GoalState::Preempted => {
                    Transition::Steps(&[State::Preempting, State::WaitingForResult])
                }
                GoalState::Succeeded => Transition::Steps(&[State::WaitingForResult]),
                GoalState::Aborted => Transition::Steps(&[State::WaitingForResult]),
                GoalState::Preempting => Transition::Steps(&[State::Preempting]),
                GoalState::Lost => Transition::UnknownStatus(GoalState::Lost),
            },
            State::WaitingForResult => match goal {
                GoalState::Pending => Transition::Invalid(self, goal),
                GoalState::Active => Transition::None,
                GoalState::Rejected => Transition::None,
                GoalState::Recalling => Transition::Invalid(self, goal),
                GoalState::Recalled => Transition::None,
                GoalState::Preempted => Transition::None,
                GoalState::Succeeded => Transition::None,
                GoalState::Aborted => Transition::None,
                GoalState::Preempting => Transition::Invalid(self, goal),
                GoalState::Lost => Transition::UnknownStatus(GoalState::Lost),
            },
            State::WaitingForCancelAck => match goal {
                GoalState::Pending => Transition::None,
                GoalState::Active => Transition::None,
                GoalState::Rejected => Transition::Steps(&[State::WaitingForResult]),
                GoalState::Recalling => Transition::Steps(&[State::Recalling]),
                GoalState::Recalled => {
                    Transition::Steps(&[State::Recalling, State::WaitingForResult])
                }
                GoalState::Preempted => {
                    Transition::Steps(&[State::Preempting, State::WaitingForResult])
                }
                GoalState::Succeeded => {
                    Transition::Steps(&[State::Preempting, State::WaitingForResult])
                }
                GoalState::Aborted => {
                    Transition::Steps(&[State::Preempting, State::WaitingForResult])
                }
                GoalState::Preempting => Transition::Steps(&[State::Preempting]),
                GoalState::Lost => Transition::UnknownStatus(GoalState::Lost),
            },
            State::Recalling => match goal {
                GoalState::Pending => Transition::Invalid(self, goal),
                GoalState::Active => Transition::Invalid(self, goal),
                GoalState::Rejected => Transition::Steps(&[State::WaitingForResult]),
                GoalState::Recalling => Transition::None,
                GoalState::Recalled => Transition::Steps(&[State::WaitingForResult]),
                GoalState::Preempted => {
                    Transition::Steps(&[State::Preempting, State::WaitingForResult])
                }
                GoalState::Succeeded => {
                    Transition::Steps(&[State::Preempting, State::WaitingForResult])
                }
                GoalState::Aborted => {
                    Transition::Steps(&[State::Preempting, State::WaitingForResult])
                }
                GoalState::Preempting => Transition::Steps(&[State::Preempting]),
                GoalState::Lost => Transition::UnknownStatus(GoalState::Lost),
            },
            State::Preempting => match goal {
                GoalState::Pending => Transition::Invalid(self, goal),
                GoalState::Active => Transition::Invalid(self, goal),
                GoalState::Rejected => Transition::Invalid(self, goal),
                GoalState::Recalling => Transition::Invalid(self, goal),
                GoalState::Recalled => Transition::Invalid(self, goal),
                GoalState::Preempted => Transition::Steps(&[State::WaitingForResult]),
                GoalState::Succeeded => Transition::Steps(&[State::WaitingForResult]),
                GoalState::Aborted => Transition::Steps(&[State::WaitingForResult]),
                GoalState::Preempting => Transition::None,
                GoalState::Lost => Transition::UnknownStatus(GoalState::Lost),
            },
            State::Done => match goal {
                GoalState::Pending => Transition::Invalid(self, goal),
                GoalState::Active => Transition::Invalid(self, goal),
                GoalState::Rejected => Transition::None,
                GoalState::Recalling => Transition::Invalid(self, goal),
                GoalState::Recalled => Transition::None,
                GoalState::Preempted => Transition::None,
                GoalState::Succeeded => Transition::None,
                GoalState::Aborted => Transition::None,
                GoalState::Preempting => Transition::Invalid(self, goal),
                GoalState::Lost => Transition::UnknownStatus(GoalState::Lost),
            },
            State::Lost => Transition::FunnyState(State::Lost),
        }
    }
}
