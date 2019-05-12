pub use crate::msg::actionlib_msgs::{GoalStatus, GoalStatusArray};
use std::convert::TryFrom;

// TODO: consider removing "Lost"
#[repr(u8)]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum GoalState {
    Pending = GoalStatus::PENDING,
    Active = GoalStatus::ACTIVE,
    Preempted = GoalStatus::PREEMPTED,
    Succeeded = GoalStatus::SUCCEEDED,
    Aborted = GoalStatus::ABORTED,
    Rejected = GoalStatus::REJECTED,
    Preempting = GoalStatus::PREEMPTING,
    Recalling = GoalStatus::RECALLING,
    Recalled = GoalStatus::RECALLED,
    Lost = GoalStatus::LOST,
}

impl TryFrom<u8> for GoalState {
    type Error = u8;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            GoalStatus::PENDING => Ok(GoalState::Pending),
            GoalStatus::ACTIVE => Ok(GoalState::Active),
            GoalStatus::PREEMPTED => Ok(GoalState::Preempted),
            GoalStatus::SUCCEEDED => Ok(GoalState::Succeeded),
            GoalStatus::ABORTED => Ok(GoalState::Aborted),
            GoalStatus::REJECTED => Ok(GoalState::Rejected),
            GoalStatus::PREEMPTING => Ok(GoalState::Preempting),
            GoalStatus::RECALLING => Ok(GoalState::Recalling),
            GoalStatus::RECALLED => Ok(GoalState::Recalled),
            GoalStatus::LOST => Ok(GoalState::Lost),
            v => Err(v),
        }
    }
}
