pub use self::action_client::{ActionClient, ClientGoalHandle, SimpleActionClient};
pub use self::action_server::ActionServer;
pub use self::goal_status::{GoalState, GoalStatus};
#[doc(hidden)]
pub use paste;
pub use rosrust_msg::actionlib_msgs::GoalID;
pub use rosrust_msg::std_msgs::Header;

pub mod action_client;
pub mod action_server;
mod goal_status;
#[macro_use]
mod macros;
mod impls;
mod static_messages;

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

#[derive(Clone, Default)]
pub struct Goal<T> {
    pub header: Header,
    pub id: GoalID,
    pub body: T,
}

pub trait ActionResponse: rosrust::Message {
    type Body: rosrust::Message;

    fn into_response(self) -> Response<Self::Body>;
    fn from_response(t: Response<Self::Body>) -> Self;
}

#[derive(Clone, Default)]
pub struct Response<T> {
    pub header: Header,
    pub status: GoalStatus,
    pub body: T,
}

type GoalBody<T> = <<T as Action>::Goal as ActionGoal>::Body;
type GoalType<T> = Goal<GoalBody<T>>;
type ResultBody<T> = <<T as Action>::Result as ActionResponse>::Body;
type ResultType<T> = Response<ResultBody<T>>;
type FeedbackBody<T> = <<T as Action>::Feedback as ActionResponse>::Body;
type FeedbackType<T> = Response<FeedbackBody<T>>;
