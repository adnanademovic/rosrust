// Long arrays as message fields cause a structure to be unable to automatically derive.
//
// Compilation of this test makes sure this is handled for those cases.

mod msg {
    rosrust::rosmsg_include!(geometry_msgs / PoseWithCovariance);
}

#[test]
fn implementations_work() {
    let mut message1 = msg::geometry_msgs::PoseWithCovariance::default();
    message1.covariance[5] = 5.0;
    let mut message2 = msg::geometry_msgs::PoseWithCovariance::default();
    message2.covariance[5] = 6.0;
    let mut message3 = msg::geometry_msgs::PoseWithCovariance::default();
    message3.covariance[5] = 5.0;
    assert_ne!(
        message1, message2,
        "Messages should not equal: {:?}, {:#?}",
        message1, message2,
    );
    assert_eq!(
        message1, message3,
        "Messages should equal: {:?}, {:#?}",
        message1, message3,
    );
}
