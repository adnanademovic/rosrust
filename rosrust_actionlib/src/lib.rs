pub use self::action_server::{
    ActionServer, ActionServerOnRequest, ActionServerState, ServerGoalHandle,
};
#[doc(hidden)]
pub use paste;

mod action_client;
mod action_server;
mod goal_id_generator;
mod goal_status;
pub mod msg;
mod status_tracker;
#[macro_use]
mod macros;

pub trait Action: rosrust::Message {
    type Goal: ActionGoal;
    type Result: ActionResponse;
    type Feedback: ActionResponse;

    fn split(self) -> (Self::Goal, Self::Result, Self::Feedback);
    fn combine(goal: Self::Goal, result: Self::Result, feedback: Self::Feedback) -> Self;
}

pub trait ActionGoal: rosrust::Message {
    type Body: rosrust::Message;

    fn into_goal(self) -> Goal<Self::Body>;
    fn from_goal(t: Goal<Self::Body>) -> Self;
}

pub struct Goal<T> {
    pub header: msg::std_msgs::Header,
    pub id: msg::actionlib_msgs::GoalID,
    pub body: T,
}

pub trait ActionResponse: rosrust::Message {
    type Body: rosrust::Message;

    fn into_response(self) -> Response<Self::Body>;
    fn from_response(t: Response<Self::Body>) -> Self;
}

pub struct Response<T> {
    pub header: msg::std_msgs::Header,
    pub status: msg::actionlib_msgs::GoalStatus,
    pub body: T,
}

type GoalBody<T> = <<T as Action>::Goal as ActionGoal>::Body;
type GoalType<T> = Goal<GoalBody<T>>;
type ResultBody<T> = <<T as Action>::Result as ActionResponse>::Body;
type ResultType<T> = Response<ResultBody<T>>;
type FeedbackBody<T> = <<T as Action>::Feedback as ActionResponse>::Body;
type FeedbackType<T> = Response<FeedbackBody<T>>;
