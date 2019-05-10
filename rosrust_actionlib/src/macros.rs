#[macro_export]
macro_rules! action {
    ($root: path; $($package:ident : $action: ident),*) => {
        $($crate::action!(INNER, $root, $package, $action);)*
    };
    (INNER, $root:path, $package: ident, $action: ident) => {
        $crate::paste::item! {
            impl $crate::Action for $root::$package::[<$action Action>] {
                type Goal = $root::$package::[<$action ActionGoal>];
                type Result = $root::$package::[<$action ActionResult>];
                type Feedback = $root::$package::[<$action ActionFeedback>];

                #[inline]
                fn split(self) -> (Self::Goal, Self::Result, Self::Feedback) {
                    (self.action_goal, self.action_result, self.action_feedback)
                }

                #[inline]
                fn combine(action_goal: Self::Goal, action_result: Self::Result, action_feedback: Self::Feedback) -> Self {
                    Self { action_goal, action_result, action_feedback }
                }
            }

            $crate::action!(ACTION_GOAL, $root, $package, $action, Goal, goal);
            $crate::action!(ACTION_RESPONSE, $root, $package, $action, Result, result);
            $crate::action!(ACTION_RESPONSE, $root, $package, $action, Feedback, feedback);
        }
    };
    (ACTION_GOAL, $root:path, $package: ident, $action: ident, $sub_message: ident, $body_key: ident) => {
        $crate::paste::item! {
            impl $crate::ActionGoal for $root::$package::[<$action Action $sub_message>] {
                type Body = $root::$package::[<$action $sub_message>];

                fn split(self) -> (
                    $crate::msg::std_msgs::Header,
                    $crate::msg::actionlib_msgs::GoalID,
                    Self::Body,
                ) {
                    let header = $crate::msg::std_msgs::Header {
                        seq: self.header.seq,
                        stamp: self.header.stamp,
                        frame_id: self.header.frame_id,
                    };
                    let goal_id = $crate::msg::actionlib_msgs::GoalID {
                        stamp: self.goal_id.stamp,
                        id: self.goal_id.id,
                    };
                    (header, goal_id, self.$body_key)
                }

                fn combine(
                    header: $crate::msg::std_msgs::Header,
                    goal_id: $crate::msg::actionlib_msgs::GoalID,
                    body: Self::Body,
                ) -> Self {
                    let header = $root::std_msgs::Header {
                        seq: header.seq,
                        stamp: header.stamp,
                        frame_id: header.frame_id,
                    };
                    let goal_id = $root::actionlib_msgs::GoalID {
                        stamp: goal_id.stamp,
                        id: goal_id.id,
                    };
                    Self { header, goal_id, $body_key: body }
                }
            }
        }
    };
    (ACTION_RESPONSE, $root:path, $package: ident, $action: ident, $sub_message: ident, $body_key: ident) => {
        $crate::paste::item! {
            impl $crate::ActionResponse for $root::$package::[<$action Action $sub_message>] {
                type Body = $root::$package::[<$action $sub_message>];

                fn split(self) -> (
                    $crate::msg::std_msgs::Header,
                    $crate::msg::actionlib_msgs::GoalStatus,
                    Self::Body,
                ) {
                    let header = $crate::msg::std_msgs::Header {
                        seq: self.header.seq,
                        stamp: self.header.stamp,
                        frame_id: self.header.frame_id,
                    };
                    let goal_id = $crate::msg::actionlib_msgs::GoalID {
                        stamp: self.status.goal_id.stamp,
                        id: self.status.goal_id.id,
                    };
                    let status = $crate::msg::actionlib_msgs::GoalStatus {
                        goal_id,
                        status: self.status.status,
                        text: self.status.text,
                    };
                    (header, status, self.$body_key)
                }

                fn combine(
                    header: $crate::msg::std_msgs::Header,
                    status: $crate::msg::actionlib_msgs::GoalStatus,
                    body: Self::Body,
                ) -> Self {
                    let header = $root::std_msgs::Header {
                        seq: header.seq,
                        stamp: header.stamp,
                        frame_id: header.frame_id,
                    };
                    let goal_id = $root::actionlib_msgs::GoalID {
                        stamp: status.goal_id.stamp,
                        id: status.goal_id.id,
                    };
                    let status = $root::actionlib_msgs::GoalStatus {
                        goal_id,
                        status: status.status,
                        text: status.text,
                    };
                    Self {
                        header,
                        status: status.into(),
                        $body_key: body,
                    }
                }
            }
        }
    };
}
