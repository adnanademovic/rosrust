mod msg {
    rosrust::rosmsg_include!(actionlib / TestAction);
}

rosrust_actionlib::action!(msg; actionlib: Test);

fn main() {}
