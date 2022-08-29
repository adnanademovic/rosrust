//! Utilities for time information based on the system clock.

/// Get the current time from the system clock.
///
/// This is esentially the same as:
/// ```
/// # let time: rosrust::Time =
/// std::time::SystemTime::now().into()
/// # ;
/// ```
///
/// # Examples
///
/// ```
/// # use ros_message::Time;
/// # #[derive(Default)]
/// # struct Header {
/// #   stamp: Time,
/// # }
/// # #[derive(Default)]
/// # struct Message {
/// #   header: Header,
/// # }
/// # let mut message = Message::default();
/// message.header.stamp = rosrust::wall_time::now();
/// ```
#[inline]
pub fn now() -> crate::Time {
    std::time::SystemTime::now().into()
}
