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

#[macro_export]
macro_rules! ros_log_once {
    ($level:expr, $($arg:tt)+) => {
        let msg = format!($($arg)*);
        $crate::log_once($level, msg, file!(), line!());
    }
}

#[macro_export]
macro_rules! ros_debug_once {
    ($($arg:tt)*) => {
        $crate::ros_log_once!($crate::msg::rosgraph_msgs::Log::DEBUG, $($arg)*);
    }
}

#[macro_export]
macro_rules! ros_info_once {
    ($($arg:tt)*) => {
        $crate::ros_log_once!($crate::msg::rosgraph_msgs::Log::INFO, $($arg)*);
    }
}

#[macro_export]
macro_rules! ros_warn_once {
    ($($arg:tt)*) => {
        $crate::ros_log_once!($crate::msg::rosgraph_msgs::Log::WARN, $($arg)*);
    }
}

#[macro_export]
macro_rules! ros_err_once {
    ($($arg:tt)*) => {
        $crate::ros_log_once!($crate::msg::rosgraph_msgs::Log::ERROR, $($arg)*);
    }
}

#[macro_export]
macro_rules! ros_fatal_once {
    ($($arg:tt)*) => {
        $crate::ros_log_once!($crate::msg::rosgraph_msgs::Log::FATAL, $($arg)*);
    }
}

#[macro_export]
macro_rules! ros_log_throttle {
    ($period:expr, $level:expr, $($arg:tt)+) => {
        let msg = format!($($arg)*);
        $crate::log_throttle($period, $level, msg, file!(), line!());
    }
}

#[macro_export]
macro_rules! ros_debug_throttle {
    ($period:expr, $($arg:tt)*) => {
        $crate::ros_log_throttle!($period, $crate::msg::rosgraph_msgs::Log::DEBUG, $($arg)*);
    }
}

#[macro_export]
macro_rules! ros_info_throttle {
    ($period:expr, $($arg:tt)*) => {
        $crate::ros_log_throttle!($period, $crate::msg::rosgraph_msgs::Log::INFO, $($arg)*);
    }
}

#[macro_export]
macro_rules! ros_warn_throttle {
    ($period:expr, $($arg:tt)*) => {
        $crate::ros_log_throttle!($period, $crate::msg::rosgraph_msgs::Log::WARN, $($arg)*);
    }
}

#[macro_export]
macro_rules! ros_err_throttle {
    ($period:expr, $($arg:tt)*) => {
        $crate::ros_log_throttle!($period, $crate::msg::rosgraph_msgs::Log::ERROR, $($arg)*);
    }
}

#[macro_export]
macro_rules! ros_fatal_throttle {
    ($period:expr, $($arg:tt)*) => {
        $crate::ros_log_throttle!($period, $crate::msg::rosgraph_msgs::Log::FATAL, $($arg)*);
    }
}
#[macro_export]
macro_rules! ros_log_throttle_identical {
    ($period:expr, $level:expr, $($arg:tt)+) => {
        let msg = format!($($arg)*);
        $crate::log_throttle_identical($period, $level, msg, file!(), line!());
    }
}

#[macro_export]
macro_rules! ros_debug_throttle_identical {
    ($period:expr, $($arg:tt)*) => {
        $crate::ros_log_throttle_identical!($period, $crate::msg::rosgraph_msgs::Log::DEBUG, $($arg)*);
    }
}

#[macro_export]
macro_rules! ros_info_throttle_identical {
    ($period:expr, $($arg:tt)*) => {
        $crate::ros_log_throttle_identical!($period, $crate::msg::rosgraph_msgs::Log::INFO, $($arg)*);
    }
}

#[macro_export]
macro_rules! ros_warn_throttle_identical {
    ($period:expr, $($arg:tt)*) => {
        $crate::ros_log_throttle_identical!($period, $crate::msg::rosgraph_msgs::Log::WARN, $($arg)*);
    }
}

#[macro_export]
macro_rules! ros_err_throttle_identical {
    ($period:expr, $($arg:tt)*) => {
        $crate::ros_log_throttle_identical!($period, $crate::msg::rosgraph_msgs::Log::ERROR, $($arg)*);
    }
}

#[macro_export]
macro_rules! ros_fatal_throttle_identical {
    ($period:expr, $($arg:tt)*) => {
        $crate::ros_log_throttle_identical!($period, $crate::msg::rosgraph_msgs::Log::FATAL, $($arg)*);
    }
}
