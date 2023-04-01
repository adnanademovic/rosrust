use rosrust_msg::actionlib_msgs::{GoalID, GoalStatus as GoalStatusRaw};
use std::convert::{TryFrom, TryInto};

// TODO: consider removing "Lost"
#[repr(u8)]
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub enum GoalState {
    #[default]
    Pending = GoalStatusRaw::PENDING,
    Active = GoalStatusRaw::ACTIVE,
    Preempted = GoalStatusRaw::PREEMPTED,
    Succeeded = GoalStatusRaw::SUCCEEDED,
    Aborted = GoalStatusRaw::ABORTED,
    Rejected = GoalStatusRaw::REJECTED,
    Preempting = GoalStatusRaw::PREEMPTING,
    Recalling = GoalStatusRaw::RECALLING,
    Recalled = GoalStatusRaw::RECALLED,
    Lost = GoalStatusRaw::LOST,
}

impl TryFrom<u8> for GoalState {
    type Error = u8;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            GoalStatusRaw::PENDING => Ok(GoalState::Pending),
            GoalStatusRaw::ACTIVE => Ok(GoalState::Active),
            GoalStatusRaw::PREEMPTED => Ok(GoalState::Preempted),
            GoalStatusRaw::SUCCEEDED => Ok(GoalState::Succeeded),
            GoalStatusRaw::ABORTED => Ok(GoalState::Aborted),
            GoalStatusRaw::REJECTED => Ok(GoalState::Rejected),
            GoalStatusRaw::PREEMPTING => Ok(GoalState::Preempting),
            GoalStatusRaw::RECALLING => Ok(GoalState::Recalling),
            GoalStatusRaw::RECALLED => Ok(GoalState::Recalled),
            GoalStatusRaw::LOST => Ok(GoalState::Lost),
            v => Err(v),
        }
    }
}

#[derive(Clone, Default)]
pub struct GoalStatus {
    pub goal_id: GoalID,
    pub state: GoalState,
    pub text: String,
}

impl From<GoalStatus> for GoalStatusRaw {
    fn from(status: GoalStatus) -> Self {
        Self {
            goal_id: status.goal_id,
            status: status.state as u8,
            text: status.text,
        }
    }
}

impl From<GoalStatusRaw> for GoalStatus {
    fn from(status: GoalStatusRaw) -> Self {
        Self {
            goal_id: status.goal_id,
            state: status.status.try_into().unwrap_or(GoalState::Lost),
            text: status.text,
        }
    }
}
