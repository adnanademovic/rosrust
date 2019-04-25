#[macro_export]
macro_rules! ros_log {
    ($level:expr, $($arg:tt)+) => {
        let msg = format!($($arg)*);
        $crate::log($level, msg, file!(), line!());
    }
}

#[macro_export]
macro_rules! ros_debug {
    ($($arg:tt)*) => {
        $crate::ros_log!($crate::msg::rosgraph_msgs::Log::DEBUG, $($arg)*);
    }
}

#[macro_export]
macro_rules! ros_info {
    ($($arg:tt)*) => {
        $crate::ros_log!($crate::msg::rosgraph_msgs::Log::INFO, $($arg)*);
    }
}

#[macro_export]
macro_rules! ros_warn {
    ($($arg:tt)*) => {
        $crate::ros_log!($crate::msg::rosgraph_msgs::Log::WARN, $($arg)*);
    }
}

#[macro_export]
macro_rules! ros_err {
    ($($arg:tt)*) => {
        $crate::ros_log!($crate::msg::rosgraph_msgs::Log::ERROR, $($arg)*);
    }
}

#[macro_export]
macro_rules! ros_fatal {
    ($($arg:tt)*) => {
        $crate::ros_log!($crate::msg::rosgraph_msgs::Log::FATAL, $($arg)*);
    }
}
