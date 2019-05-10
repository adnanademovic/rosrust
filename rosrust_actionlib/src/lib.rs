pub use self::goal_id_generator::GoalIdGenerator;
#[doc(hidden)]
pub use paste;

mod goal_id_generator;
#[macro_use]
mod macros;
pub mod msg;

pub trait Action: rosrust::Message {
    type Goal: ActionGoal;
    type Result: ActionResponse;
    type Feedback: ActionResponse;

    fn split(self) -> (Self::Goal, Self::Result, Self::Feedback);
    fn combine(goal: Self::Goal, result: Self::Result, feedback: Self::Feedback) -> Self;
}

pub trait ActionGoal: rosrust::Message {
    type Body: rosrust::Message;

    fn split(
        self,
    ) -> (
        msg::std_msgs::Header,
        msg::actionlib_msgs::GoalID,
        Self::Body,
    );
    fn combine(
        header: msg::std_msgs::Header,
        id: msg::actionlib_msgs::GoalID,
        body: Self::Body,
    ) -> Self;
}

pub trait ActionResponse: rosrust::Message {
    type Body: rosrust::Message;

    fn split(
        self,
    ) -> (
        msg::std_msgs::Header,
        msg::actionlib_msgs::GoalStatus,
        Self::Body,
    );
    fn combine(
        header: msg::std_msgs::Header,
        status: msg::actionlib_msgs::GoalStatus,
        body: Self::Body,
    ) -> Self;
}
