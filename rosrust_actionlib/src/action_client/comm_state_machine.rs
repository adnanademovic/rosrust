use crate::goal_status::{GoalID, GoalState, GoalStatus, GoalStatusArray};
use crate::{Action, FeedbackBody, FeedbackType, GoalType, ResultType};

type OnFeedback<T> = Box<dyn Fn(FeedbackBody<T>) + Send + Sync + 'static>;
type OnTransition = Box<dyn Fn() + Send + Sync + 'static>;

pub struct CommStateMachine<T: Action> {
    action_goal: GoalType<T>,
    on_feedback: Option<OnFeedback<T>>,
    on_transition: Option<OnTransition>,
    send_goal_handler: Box<dyn Fn() + Send + Sync + 'static>,
    send_cancel_handler: Box<dyn Fn(GoalID) + Send + Sync + 'static>,
    state: State,
    latest_goal_status: GoalStatus,
    latest_result: Option<ResultType<T>>,
}

impl<T: Action> CommStateMachine<T> {
    pub(crate) fn new(
        action_goal: GoalType<T>,
        on_feedback: Option<OnFeedback<T>>,
        on_transition: Option<OnTransition>,
    ) -> Self {
        Self {
            action_goal,
            on_feedback,
            on_transition,
            send_goal_handler: Box::new(|| {}),
            send_cancel_handler: Box::new(|_| {}),
            state: State::WaitingForGoalAck,
            latest_goal_status: GoalStatus {
                status: GoalState::Pending as u8,
                ..Default::default()
            },
            latest_result: None,
        }
    }

    pub(crate) fn register_send_goal<F: Fn() + Send + Sync + 'static>(&mut self, f: F) {
        self.send_goal_handler = Box::new(f);
    }

    pub(crate) fn register_send_cancel<F: Fn(GoalID) + Send + Sync + 'static>(&mut self, f: F) {
        self.send_cancel_handler = Box::new(f);
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
        self.state
    }

    #[inline]
    pub fn latest_goal_status(&self) -> &GoalStatus {
        &self.latest_goal_status
    }

    #[inline]
    pub fn latest_result(&self) -> &Option<ResultType<T>> {
        &self.latest_result
    }

    #[inline]
    pub fn set_state(&mut self, state: State) {
        rosrust::ros_debug!(
            "Transitioning client State from {:?} to {:?}",
            self.state,
            state
        );
        self.state = state;
    }

    pub fn transition_to(&mut self, state: State) {
        rosrust::ros_debug!(
            "Transitioning to {:?} (from {:?}, goal: {})",
            state,
            self.state,
            self.action_goal.id.id,
        );

        self.state = state;

        if let Some(on_transition) = &self.on_transition {
            on_transition()
        }
    }

    pub fn update_feedback(&self, action_feedback: FeedbackType<T>) {
        if self.action_goal.id.id != action_feedback.status.goal_id.id {
            return;
        }

        if self.state == State::Done {
            return;
        }

        if let Some(on_feedback) = &self.on_feedback {
            on_feedback(action_feedback.body)
        }
    }

    pub fn update_result(&mut self, action_result: ResultType<T>) {
        if self.action_goal.id.id != action_result.status.goal_id.id {
            return;
        }

        let status = action_result.status.clone();

        self.latest_goal_status = status.clone();
        self.latest_result = Some(action_result);

        match self.state {
            State::WaitingForGoalAck
            | State::Pending
            | State::Active
            | State::WaitingForResult
            | State::WaitingForCancelAck
            | State::Recalling
            | State::Preempting => {
                let mut status_array = GoalStatusArray::default();
                status_array.status_list.push(status);
                self.update_status(status_array);
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

    fn update_status_inner(&mut self, status_array: GoalStatusArray) -> Result<(), Option<String>> {
        use std::convert::TryInto;

        if self.state == State::Done {
            return Err(None);
        }
        let status = status_array
            .status_list
            .into_iter()
            .find(|status| status.goal_id.id == self.action_goal.id.id)
            .ok_or_else(|| {
                match self.state {
                    State::Pending
                    | State::Active
                    | State::WaitingForCancelAck
                    | State::Recalling
                    | State::Preempting
                    | State::Lost => self.mark_as_lost(),
                    State::WaitingForGoalAck | State::WaitingForResult | State::Done => {}
                }
                return None;
            })?;

        let goal_state = status
            .status
            .try_into()
            .map_err(|err| format!("Got an unknown status from the ActionServer: {}", err))?;

        self.latest_goal_status = status;

        let steps = self
            .state
            .transition_to(goal_state)
            .into_update_status_steps()?;

        for step in steps {
            self.transition_to(*step);
        }

        Ok(())
    }

    pub fn update_status(&mut self, status_array: GoalStatusArray) {
        if let Err(Some(err)) = self.update_status_inner(status_array) {
            rosrust::ros_err!("{}", err);
        }
    }

    fn mark_as_lost(&mut self) {
        self.latest_goal_status.status = GoalState::Lost as u8;
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
    NoTransition,
    Invalid(State, GoalState),
    FunnyState(State),
    UnknownStatus(GoalState),
    Steps(&'static [State]),
}

impl Transition {
    fn into_update_status_steps(self) -> Result<&'static [State], String> {
        match self {
            Transition::NoTransition => Ok(&[]),
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
                GoalState::Pending => Transition::NoTransition,
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
                GoalState::Active => Transition::NoTransition,
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
                GoalState::Active => Transition::NoTransition,
                GoalState::Rejected => Transition::NoTransition,
                GoalState::Recalling => Transition::Invalid(self, goal),
                GoalState::Recalled => Transition::NoTransition,
                GoalState::Preempted => Transition::NoTransition,
                GoalState::Succeeded => Transition::NoTransition,
                GoalState::Aborted => Transition::NoTransition,
                GoalState::Preempting => Transition::Invalid(self, goal),
                GoalState::Lost => Transition::UnknownStatus(GoalState::Lost),
            },
            State::WaitingForCancelAck => match goal {
                GoalState::Pending => Transition::NoTransition,
                GoalState::Active => Transition::NoTransition,
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
                GoalState::Recalling => Transition::NoTransition,
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
                GoalState::Preempting => Transition::NoTransition,
                GoalState::Lost => Transition::UnknownStatus(GoalState::Lost),
            },
            State::Done => match goal {
                GoalState::Pending => Transition::Invalid(self, goal),
                GoalState::Active => Transition::Invalid(self, goal),
                GoalState::Rejected => Transition::NoTransition,
                GoalState::Recalling => Transition::Invalid(self, goal),
                GoalState::Recalled => Transition::NoTransition,
                GoalState::Preempted => Transition::NoTransition,
                GoalState::Succeeded => Transition::NoTransition,
                GoalState::Aborted => Transition::NoTransition,
                GoalState::Preempting => Transition::Invalid(self, goal),
                GoalState::Lost => Transition::UnknownStatus(GoalState::Lost),
            },
            State::Lost => Transition::FunnyState(State::Lost),
        }
    }
}
