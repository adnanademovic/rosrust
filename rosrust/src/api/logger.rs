#[macro_export]
macro_rules! ros_log {
    ($ros:ident, $level:expr, $msg:expr) => {
        $ros.log($level, &$msg, file!(), line!());
    }
}

#[macro_export]
macro_rules! ros_debug {
    ($ros:ident, $msg:expr) => {
        ros_log!($ros, $crate::msg::rosgraph_msgs::Log::DEBUG, $msg);
    }
}

#[macro_export]
macro_rules! ros_info {
    ($ros:ident, $msg:expr) => {
        ros_log!($ros, $crate::msg::rosgraph_msgs::Log::INFO, $msg);
    }
}

#[macro_export]
macro_rules! ros_warn {
    ($ros:ident, $msg:expr) => {
        ros_log!($ros, $crate::msg::rosgraph_msgs::Log::WARN, $msg);
    }
}

#[macro_export]
macro_rules! ros_err {
    ($ros:ident, $msg:expr) => {
        ros_log!($ros, $crate::msg::rosgraph_msgs::Log::ERROR, $msg);
    }
}

#[macro_export]
macro_rules! ros_fatal {
    ($ros:ident, $msg:expr) => {
        ros_log!($ros, $crate::msg::rosgraph_msgs::Log::FATAL, $msg);
    }
}
