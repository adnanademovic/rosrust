#[macro_export]
macro_rules! ros_log {
    ($level:expr, $($arg:tt)+) => {
        #[allow(unknown_lints, useless_format)]
        let msg = format!($($arg)*);
        match $level {
            $crate::msg::rosgraph_msgs::Log::DEBUG => {
                println!("[DEBUG @ {}:{}]: {}", file!(), line!(), msg);
            }
            $crate::msg::rosgraph_msgs::Log::INFO => {
                println!("[INFO @ {}:{}]: {}", file!(), line!(), msg);
            }
            $crate::msg::rosgraph_msgs::Log::WARN => {
                eprintln!("[WARN @ {}:{}]: {}", file!(), line!(), msg);

            }
            $crate::msg::rosgraph_msgs::Log::ERROR => {
                eprintln!("[ERROR @ {}:{}]: {}", file!(), line!(), msg);

            }
            $crate::msg::rosgraph_msgs::Log::FATAL => {
                eprintln!("[FATAL @ {}:{}]: {}", file!(), line!(), msg);
            }
            _ => {}
        }
        $crate::log($level, msg, file!(), line!());
    }
}

#[macro_export]
macro_rules! ros_debug {
    ($($arg:tt)*) => {
        ros_log!($crate::msg::rosgraph_msgs::Log::DEBUG, $($arg)*);
    }
}

#[macro_export]
macro_rules! ros_info {
    ($($arg:tt)*) => {
        ros_log!($crate::msg::rosgraph_msgs::Log::INFO, $($arg)*);
    }
}

#[macro_export]
macro_rules! ros_warn {
    ($($arg:tt)*) => {
        ros_log!($crate::msg::rosgraph_msgs::Log::WARN, $($arg)*);
    }
}

#[macro_export]
macro_rules! ros_err {
    ($($arg:tt)*) => {
        ros_log!($crate::msg::rosgraph_msgs::Log::ERROR, $($arg)*);
    }
}

#[macro_export]
macro_rules! ros_fatal {
    ($($arg:tt)*) => {
        ros_log!($crate::msg::rosgraph_msgs::Log::FATAL, $($arg)*);
    }
}
