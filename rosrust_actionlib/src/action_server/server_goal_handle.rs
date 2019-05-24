use super::{publish_response, StatusList, StatusTracker};
use crate::static_messages::MUTEX_LOCK_FAIL;
use crate::{Action, FeedbackBody, GoalBody, GoalID, GoalState, GoalStatus, GoalType, ResultBody};
use std::sync::{Arc, Mutex};

pub struct ServerGoalHandle<T: Action> {
    result_pub: rosrust::Publisher<T::Result>,
    feedback_pub: rosrust::Publisher<T::Feedback>,
    goal: Arc<GoalType<T>>,
    status_list: Arc<Mutex<StatusList<T>>>,
    status_tracker: Arc<Mutex<StatusTracker<GoalBody<T>>>>,
}

impl<T: Action> std::cmp::PartialEq<Self> for ServerGoalHandle<T> {
    fn eq(&self, other: &Self) -> bool {
        self.goal_id().id == other.goal_id().id
    }
}

impl<T: Action> ServerGoalHandle<T> {
    pub(crate) fn new(
        result_pub: rosrust::Publisher<T::Result>,
        feedback_pub: rosrust::Publisher<T::Feedback>,
        goal: Arc<GoalType<T>>,
        status_list: Arc<Mutex<StatusList<T>>>,
        status_tracker: Arc<Mutex<StatusTracker<GoalBody<T>>>>,
    ) -> rosrust::error::Result<Self> {
        Ok(Self {
            result_pub,
            feedback_pub,
            goal,
            status_list,
            status_tracker,
        })
    }

    fn log_action(&self, operation: &str) {
        let goal_id = self.goal_id();
        let id = goal_id.id;
        let stamp = goal_id.stamp.seconds();
        rosrust::ros_debug!("{}, id: {}, stamp: {}", operation, id, stamp);
    }

    #[inline]
    fn set_general(
        &self,
        result: Option<ResultBody<T>>,
        text: Option<&str>,
        action: ActionCommand,
    ) -> Result<(), Error> {
        self.log_action(action.precise_description());

        let mut status_tracker = self.status_tracker.lock().expect(MUTEX_LOCK_FAIL);

        let new_tracker_state = action
            .do_transition(status_tracker.state())
            .map_err(Into::into)?;
        status_tracker.set_state(new_tracker_state);

        if let Some(text) = text {
            status_tracker.set_text(text);
        }

        match action.publish_target() {
            ActionPublishTarget::Status => {
                drop(status_tracker);

                self.status_list
                    .lock()
                    .expect(MUTEX_LOCK_FAIL)
                    .publish()
                    .map_err(|err| Error::new(format!("Failed to publish status: {}", err)))
            }
            ActionPublishTarget::Result => {
                status_tracker.mark_for_destruction(true);

                let status = status_tracker.to_status();
                drop(status_tracker);

                publish_response(&self.result_pub, status, result.unwrap_or_default())
                    .map_err(|err| Error::new(format!("Failed to publish result: {}", err)))
            }
        }
    }

    pub fn response(&self) -> ServerGoalHandleMessageBuilder<T> {
        ServerGoalHandleMessageBuilder {
            gh: self,
            text: "",
            result: None,
        }
    }

    fn set_public(&self, result: Option<ResultBody<T>>, text: &str, action: ActionCommand) -> bool {
        self.set_general(result, Some(text), action)
            .map_err(Error::log)
            .is_ok()
    }

    pub(crate) fn set_cancel_requested(&self) -> bool {
        // We intentionally do not log the error for this case
        self.set_general(None, None, ActionCommand::CancelRequested)
            .is_ok()
    }

    fn publish_feedback_inner(&self, feedback: FeedbackBody<T>) -> Result<(), Error> {
        self.log_action("Publishing feedback on goal");

        let status = self
            .status_tracker
            .lock()
            .expect(MUTEX_LOCK_FAIL)
            .to_status();

        publish_response(&self.feedback_pub, status, feedback)
            .map_err(|err| Error::new(format!("Failed to publish feedback: {}", err)))
    }

    pub fn publish_feedback(&self, feedback: FeedbackBody<T>) -> bool {
        self.publish_feedback_inner(feedback)
            .map_err(Error::log)
            .is_ok()
    }

    pub fn goal_message(&self) -> Arc<GoalType<T>> {
        self.goal.clone()
    }

    pub fn goal(&self) -> &GoalBody<T> {
        &self.goal.body
    }

    pub fn goal_id(&self) -> GoalID {
        self.status_tracker
            .lock()
            .expect(MUTEX_LOCK_FAIL)
            .goal_id()
            .clone()
    }

    pub fn goal_status(&self) -> GoalStatus {
        self.status_tracker
            .lock()
            .expect(MUTEX_LOCK_FAIL)
            .to_status()
    }
}

pub struct ServerGoalHandleMessageBuilder<'a, T: Action> {
    gh: &'a ServerGoalHandle<T>,
    text: &'a str,
    result: Option<ResultBody<T>>,
}

impl<'a, T: Action> ServerGoalHandleMessageBuilder<'a, T> {
    #[inline]
    pub fn text(&mut self, text: &'a str) -> &mut Self {
        self.text = text;
        self
    }

    #[inline]
    pub fn result(&mut self, result: ResultBody<T>) -> &mut Self {
        self.result = Some(result);
        self
    }

    #[inline]
    pub fn send_accepted(&mut self) -> bool {
        self.gh.set_public(None, self.text, ActionCommand::Accepted)
    }

    #[inline]
    pub fn send_canceled(&mut self) -> bool {
        self.gh
            .set_public(self.result.take(), self.text, ActionCommand::Canceled)
    }

    #[inline]
    pub fn send_rejected(&mut self) -> bool {
        self.gh
            .set_public(self.result.take(), self.text, ActionCommand::Rejected)
    }

    #[inline]
    pub fn send_aborted(&mut self) -> bool {
        self.gh
            .set_public(self.result.take(), self.text, ActionCommand::Aborted)
    }

    #[inline]
    pub fn send_succeeded(&mut self) -> bool {
        self.gh
            .set_public(self.result.take(), self.text, ActionCommand::Succeeded)
    }
}

struct Error {
    message: String,
}

impl Error {
    fn new(message: String) -> Self {
        Self { message }
    }

    fn log(self) -> Self {
        rosrust::ros_err!("{}", self.message);
        self
    }
}

struct TransitionIssue {
    target: ActionCommand,
    accepted: &'static [GoalState],
    status: GoalState,
}

impl std::fmt::Display for TransitionIssue {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "To transition to {} state, the goal must be in a ",
            match self.target {
                ActionCommand::Accepted => "an active",
                ActionCommand::Canceled => "a canceled",
                ActionCommand::Rejected => "a rejected",
                ActionCommand::Aborted => "an aborted",
                ActionCommand::Succeeded => "a succeeded",
                ActionCommand::CancelRequested => "a requested cancel",
            }
        )?;

        match self.accepted.split_last() {
            None => write!(f, "nonexistent")?,
            Some((item, [])) => write!(f, "{:?}", item)?,
            Some((item2, [item1])) => write!(f, "{:?} or {:?}", item1, item2)?,
            Some((last, most)) => {
                for item in most {
                    write!(f, "{:?}, ", item)?;
                }
                write!(f, "or {:?}", last)?;
            }
        }

        write!(f, " state, it is currently in state: {:?}", self.status)?;
        Ok(())
    }
}

#[derive(Copy, Clone, Debug)]
enum ActionCommand {
    Accepted,
    Canceled,
    Rejected,
    Aborted,
    Succeeded,
    CancelRequested,
}

enum ActionPublishTarget {
    Status,
    Result,
}

impl ActionCommand {
    fn publish_target(self) -> ActionPublishTarget {
        match self {
            ActionCommand::Accepted | ActionCommand::CancelRequested => ActionPublishTarget::Status,
            ActionCommand::Canceled
            | ActionCommand::Rejected
            | ActionCommand::Aborted
            | ActionCommand::Succeeded => ActionPublishTarget::Result,
        }
    }

    fn precise_description(self) -> &'static str {
        match self {
            ActionCommand::Accepted => "Accepting goal",
            ActionCommand::Canceled => "Setting status to canceled on goal",
            ActionCommand::Rejected => "Setting status to rejected on goal",
            ActionCommand::Aborted => "Setting status to aborted on goal",
            ActionCommand::Succeeded => "Setting status to succeeded on goal",
            ActionCommand::CancelRequested => "Transitioning to a cancel requested state on goal",
        }
    }

    fn accepted_states(self) -> &'static [GoalState] {
        match self {
            ActionCommand::Accepted => &[GoalState::Pending, GoalState::Recalling],
            ActionCommand::Canceled => &[
                GoalState::Pending,
                GoalState::Recalling,
                GoalState::Active,
                GoalState::Preempting,
            ],
            ActionCommand::Rejected => &[GoalState::Pending, GoalState::Recalling],
            ActionCommand::Aborted => &[GoalState::Preempting, GoalState::Active],
            ActionCommand::Succeeded => &[GoalState::Preempting, GoalState::Active],
            ActionCommand::CancelRequested => &[GoalState::Pending, GoalState::Active],
        }
    }

    fn do_transition(self, state: GoalState) -> Result<GoalState, Error> {
        match (self, state) {
            (ActionCommand::Accepted, GoalState::Pending) => Ok(GoalState::Active),
            (ActionCommand::Accepted, GoalState::Recalling) => Ok(GoalState::Preempting),

            (ActionCommand::Canceled, GoalState::Pending)
            | (ActionCommand::Canceled, GoalState::Recalling) => Ok(GoalState::Recalled),
            (ActionCommand::Canceled, GoalState::Active)
            | (ActionCommand::Canceled, GoalState::Preempting) => Ok(GoalState::Preempted),

            (ActionCommand::Rejected, GoalState::Pending)
            | (ActionCommand::Rejected, GoalState::Recalling) => Ok(GoalState::Rejected),

            (ActionCommand::Aborted, GoalState::Preempting)
            | (ActionCommand::Aborted, GoalState::Active) => Ok(GoalState::Aborted),

            (ActionCommand::Succeeded, GoalState::Preempting)
            | (ActionCommand::Succeeded, GoalState::Active) => Ok(GoalState::Succeeded),

            (ActionCommand::CancelRequested, GoalState::Pending) => Ok(GoalState::Recalling),
            (ActionCommand::CancelRequested, GoalState::Active) => Ok(GoalState::Preempting),

            (target, status) => {
                let issue = TransitionIssue {
                    accepted: target.accepted_states(),
                    target,
                    status,
                };
                Err(Error::new(format!("{}", issue)))
            }
        }
    }
}
