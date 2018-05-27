#[macro_export]
macro_rules! rosmsg_include {
    ($msgs:expr) => {
        mod __rosrust_rosmsg_include {
            #[derive(RosmsgInclude)]
            #[rosmsg_includes=$msgs]
            struct _RosmsgIncludeDummy;
        }
        pub use self::__rosrust_rosmsg_include::*;
    };
}

mod __rosrust_rosmsg_include {
    #[derive(RosmsgInclude)]
    #[rosmsg_includes = "rosgraph_msgs/Clock, rosgraph_msgs/Log"]
    #[rosrust_internal]
    struct _RosmsgIncludeDummy;
}
pub use self::__rosrust_rosmsg_include::*;
