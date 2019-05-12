use crate::goal_status::{GoalState, GoalStatus, GoalStatusArray};
use crate::{Action, FeedbackBody, FeedbackType, GoalType, ResultType};

type OnFeedback<T> = Box<Fn(ClientGoalHandle, FeedbackBody<T>)>;
type OnTransition = Box<Fn(ClientGoalHandle)>;
type SendGoal = Box<Fn()>;
type SendCancel = Box<Fn()>;

pub struct ClientGoalHandle;

pub struct CommStateMachine<T: Action> {
    action_goal: GoalType<T>,
    on_feedback: Option<OnFeedback<T>>,
    on_transition: Option<OnTransition>,
    send_goal: SendGoal,
    send_cancel: SendCancel,
    state: State,
    latest_goal_status: GoalStatus,
    latest_result: Option<ResultType<T>>,
}

impl<T: Action> CommStateMachine<T> {
    pub fn new(
        action_goal: GoalType<T>,
        on_feedback: Option<OnFeedback<T>>,
        on_transition: Option<OnTransition>,
        send_goal: SendGoal,
        send_cancel: SendCancel,
    ) -> Self {
        Self {
            action_goal,
            on_feedback,
            on_transition,
            send_goal,
            send_cancel,
            state: State::WaitingForGoalAck,
            latest_goal_status: GoalStatus {
                status: GoalState::Pending as u8,
                ..Default::default()
            },
            latest_result: None,
        }
    }

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
            on_transition(ClientGoalHandle);
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
            on_feedback(ClientGoalHandle, action_feedback.body)
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

    // TODO: implement using a helper function that returns a result to be displayed in log
    pub fn update_status(&mut self, status_array: GoalStatusArray) {
        use std::convert::TryInto;

        if self.state == State::Done {
            return;
        }
        let status_option = status_array
            .status_list
            .into_iter()
            .find(|status| status.goal_id.id == self.action_goal.id.id);

        let status = match status_option {
            Some(status) => status,
            None => {
                match self.state {
                    State::Pending
                    | State::Active
                    | State::WaitingForCancelAck
                    | State::Recalling
                    | State::Preempting
                    | State::Lost => self.mark_as_lost(),
                    State::WaitingForGoalAck | State::WaitingForResult | State::Done => {}
                }
                return;
            }
        };

        let goal_state = match status.status.try_into() {
            Ok(s) => s,
            Err(err) => {
                rosrust::ros_err!("Got an unknown status from the ActionServer: {}", err);
                return;
            }
        };
        self.latest_goal_status = status;

        match self.state.transition_to(goal_state) {
            Transition::NoTransition => {}
            Transition::Invalid => {
                rosrust::ros_err!(
                    "Invalid goal status transition from {:?} to {:?}",
                    self.state,
                    goal_state,
                );
            }
            Transition::FunnyState(state) => {
                rosrust::ros_err!("CommStateMachine is in a funny state: {:?}", state);
            }
            Transition::UnknownStatus(goal_status) => {
                rosrust::ros_err!(
                    "Got an unknown status from the ActionServer: {:?}",
                    goal_status
                );
            }
            Transition::Step(step) => self.transition_to(step),
            Transition::Steps(steps) => {
                for step in steps {
                    self.transition_to(*step);
                }
            }
        };
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
    Invalid,
    FunnyState(State),
    UnknownStatus(GoalState),
    Step(State),
    Steps(&'static [State]),
}

impl State {
    pub fn transition_to(self, goal: GoalState) -> Transition {
        match self {
            State::WaitingForGoalAck => match goal {
                GoalState::Pending => Transition::Step(State::Pending),
                GoalState::Active => Transition::Step(State::Active),
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
                GoalState::Active => Transition::Step(State::Active),
                GoalState::Rejected => Transition::Step(State::WaitingForResult),
                GoalState::Recalling => Transition::Step(State::Recalling),
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
                GoalState::Pending => Transition::Invalid,
                GoalState::Active => Transition::NoTransition,
                GoalState::Rejected => Transition::Invalid,
                GoalState::Recalling => Transition::Invalid,
                GoalState::Recalled => Transition::Invalid,
                GoalState::Preempted => {
                    Transition::Steps(&[State::Preempting, State::WaitingForResult])
                }
                GoalState::Succeeded => Transition::Step(State::WaitingForResult),
                GoalState::Aborted => Transition::Step(State::WaitingForResult),
                GoalState::Preempting => Transition::Step(State::Preempting),
                GoalState::Lost => Transition::UnknownStatus(GoalState::Lost),
            },
            State::WaitingForResult => match goal {
                GoalState::Pending => Transition::Invalid,
                GoalState::Active => Transition::NoTransition,
                GoalState::Rejected => Transition::NoTransition,
                GoalState::Recalling => Transition::Invalid,
                GoalState::Recalled => Transition::NoTransition,
                GoalState::Preempted => Transition::NoTransition,
                GoalState::Succeeded => Transition::NoTransition,
                GoalState::Aborted => Transition::NoTransition,
                GoalState::Preempting => Transition::Invalid,
                GoalState::Lost => Transition::UnknownStatus(GoalState::Lost),
            },
            State::WaitingForCancelAck => match goal {
                GoalState::Pending => Transition::NoTransition,
                GoalState::Active => Transition::NoTransition,
                GoalState::Rejected => Transition::Step(State::WaitingForResult),
                GoalState::Recalling => Transition::Step(State::Recalling),
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
                GoalState::Preempting => Transition::Step(State::Preempting),
                GoalState::Lost => Transition::UnknownStatus(GoalState::Lost),
            },
            State::Recalling => match goal {
                GoalState::Pending => Transition::Invalid,
                GoalState::Active => Transition::Invalid,
                GoalState::Rejected => Transition::Step(State::WaitingForResult),
                GoalState::Recalling => Transition::NoTransition,
                GoalState::Recalled => Transition::Step(State::WaitingForResult),
                GoalState::Preempted => {
                    Transition::Steps(&[State::Preempting, State::WaitingForResult])
                }
                GoalState::Succeeded => {
                    Transition::Steps(&[State::Preempting, State::WaitingForResult])
                }
                GoalState::Aborted => {
                    Transition::Steps(&[State::Preempting, State::WaitingForResult])
                }
                GoalState::Preempting => Transition::Step(State::Preempting),
                GoalState::Lost => Transition::UnknownStatus(GoalState::Lost),
            },
            State::Preempting => match goal {
                GoalState::Pending => Transition::Invalid,
                GoalState::Active => Transition::Invalid,
                GoalState::Rejected => Transition::Invalid,
                GoalState::Recalling => Transition::Invalid,
                GoalState::Recalled => Transition::Invalid,
                GoalState::Preempted => Transition::Step(State::WaitingForResult),
                GoalState::Succeeded => Transition::Step(State::WaitingForResult),
                GoalState::Aborted => Transition::Step(State::WaitingForResult),
                GoalState::Preempting => Transition::NoTransition,
                GoalState::Lost => Transition::UnknownStatus(GoalState::Lost),
            },
            State::Done => match goal {
                GoalState::Pending => Transition::Invalid,
                GoalState::Active => Transition::Invalid,
                GoalState::Rejected => Transition::NoTransition,
                GoalState::Recalling => Transition::Invalid,
                GoalState::Recalled => Transition::NoTransition,
                GoalState::Preempted => Transition::NoTransition,
                GoalState::Succeeded => Transition::NoTransition,
                GoalState::Aborted => Transition::NoTransition,
                GoalState::Preempting => Transition::Invalid,
                GoalState::Lost => Transition::UnknownStatus(GoalState::Lost),
            },
            State::Lost => Transition::FunnyState(State::Lost),
        }
    }
}
