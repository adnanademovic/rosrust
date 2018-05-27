#[macro_use]
extern crate rosrust_codegen;

#[test]
fn compiles() {
    #[derive(RosmsgInclude)]
    #[rosmsg_includes = "std_msgs/String, roscpp_tutorials/TwoInts"]
    struct RosmsgIncludeDummy;
}
