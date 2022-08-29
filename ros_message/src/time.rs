use serde_derive::{Deserialize, Serialize};
use std::cmp;
use std::fmt;
use std::fmt::Formatter;
use std::hash::{Hash, Hasher};
use std::ops;
use std::time;

const BILLION: i64 = 1_000_000_000;

/// ROS representation of time, with nanosecond precision
#[derive(Copy, Clone, Default, Serialize, Deserialize, Debug, Eq)]
pub struct Time {
    /// Number of seconds.
    pub sec: u32,
    /// Number of nanoseconds inside the current second.
    pub nsec: u32,
}

impl Hash for Time {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.nanos().hash(state)
    }
}

impl Time {
    /// Creates a new time of zero value.
    ///
    /// # Examples
    ///
    /// ```
    /// # use ros_message::Time;
    /// assert_eq!(Time::new(), Time { sec: 0, nsec: 0 });
    /// ```
    #[inline]
    pub fn new() -> Time {
        Self::default()
    }

    /// Creates a time of the given number of nanoseconds.
    ///
    /// # Examples
    ///
    /// ```
    /// # use ros_message::Time;
    /// assert_eq!(Time::from_nanos(0), Time { sec: 0, nsec: 0 });
    /// assert_eq!(Time::from_nanos(12_000_000_123), Time { sec: 12, nsec: 123 });
    /// ```
    #[inline]
    pub fn from_nanos(t: i64) -> Time {
        Time {
            sec: (t / BILLION) as u32,
            nsec: (t % BILLION) as u32,
        }
    }

    /// Creates a time of the given number of seconds.
    ///
    /// # Examples
    ///
    /// ```
    /// # use ros_message::Time;
    /// assert_eq!(Time::from_seconds(0), Time { sec: 0, nsec: 0 });
    /// assert_eq!(Time::from_seconds(12), Time { sec: 12, nsec: 0 });
    /// ```
    #[inline]
    pub fn from_seconds(sec: u32) -> Time {
        Time { sec, nsec: 0 }
    }

    /// Returns the number of nanoseconds in the time.
    ///
    /// # Examples
    ///
    /// ```
    /// # use ros_message::Time;
    /// assert_eq!(Time { sec: 0, nsec: 0 }.nanos(), 0);
    /// assert_eq!(Time { sec: 12, nsec: 123 }.nanos(), 12_000_000_123);
    /// ```
    #[inline]
    pub fn nanos(self) -> i64 {
        i64::from(self.sec) * BILLION + i64::from(self.nsec)
    }

    /// Returns the number of seconds in the time.
    ///
    /// # Examples
    ///
    /// ```
    /// # use ros_message::Time;
    /// assert_eq!(Time { sec: 0, nsec: 0 }.seconds(), 0.0);
    /// assert_eq!(Time { sec: 12, nsec: 123 }.seconds(), 12.000_000_123);
    /// ```
    #[inline]
    pub fn seconds(self) -> f64 {
        f64::from(self.sec) + f64::from(self.nsec) / BILLION as f64
    }
}

fn display_nanos(nanos: &str, f: &mut Formatter<'_>) -> fmt::Result {
    // Special display function to handle edge cases like
    // Duration { sec: -1, nsec: 1 } and Duration { sec: -1, nsec: -1 }
    let split_point = nanos.len() - 9;
    let characters = nanos.chars();
    let (left, right) = characters.as_str().split_at(split_point);
    write!(f, "{}.{}", left, right)
}

impl fmt::Display for Time {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        display_nanos(&format!("{:010}", self.nanos()), f)
    }
}

impl cmp::PartialEq for Time {
    fn eq(&self, other: &Self) -> bool {
        self.nanos() == other.nanos()
    }
}

impl cmp::PartialOrd for Time {
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        self.nanos().partial_cmp(&other.nanos())
    }
}

impl cmp::Ord for Time {
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        self.nanos().cmp(&other.nanos())
    }
}

/// ROS representation of duration, with nanosecond precision
#[derive(Copy, Clone, Default, Serialize, Deserialize, Debug, Eq)]
pub struct Duration {
    /// Number of seconds. Negative for negative durations.
    pub sec: i32,
    /// Number of nanoseconds inside the current second. Negative for negative durations.
    pub nsec: i32,
}

impl Hash for Duration {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.nanos().hash(state)
    }
}

impl fmt::Display for Duration {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let data = self.nanos();
        display_nanos(
            &format!(
                "{}{:010}",
                if data.is_negative() { "-" } else { "" },
                data.abs()
            ),
            f,
        )
    }
}

impl Duration {
    /// Creates a new duration of zero value.
    ///
    /// # Examples
    ///
    /// ```
    /// # use ros_message::Duration;
    /// assert_eq!(Duration::new(), Duration { sec: 0, nsec: 0 });
    /// ```
    #[inline]
    pub fn new() -> Duration {
        Self::default()
    }

    /// Creates a duration of the given number of nanoseconds.
    ///
    /// # Examples
    ///
    /// ```
    /// # use ros_message::Duration;
    /// assert_eq!(Duration::from_nanos(0), Duration { sec: 0, nsec: 0 });
    /// assert_eq!(Duration::from_nanos(12_000_000_123), Duration { sec: 12, nsec: 123 });
    /// assert_eq!(Duration::from_nanos(-12_000_000_123), Duration { sec: -12, nsec: -123 });
    /// ```
    #[inline]
    pub fn from_nanos(t: i64) -> Duration {
        Duration {
            sec: (t / BILLION) as i32,
            nsec: (t % BILLION) as i32,
        }
    }

    /// Creates a duration of the given number of seconds.
    ///
    /// # Examples
    ///
    /// ```
    /// # use ros_message::Duration;
    /// assert_eq!(Duration::from_seconds(0), Duration { sec: 0, nsec: 0 });
    /// assert_eq!(Duration::from_seconds(12), Duration { sec: 12, nsec: 0 });
    /// assert_eq!(Duration::from_seconds(-12), Duration { sec: -12, nsec: 0 });
    /// ```
    #[inline]
    pub fn from_seconds(sec: i32) -> Duration {
        Duration { sec, nsec: 0 }
    }

    /// Returns the number of nanoseconds in the duration.
    ///
    /// # Examples
    ///
    /// ```
    /// # use ros_message::Duration;
    /// assert_eq!(Duration { sec: 0, nsec: 0 }.nanos(), 0);
    /// assert_eq!(Duration { sec: 12, nsec: 123 }.nanos(), 12_000_000_123);
    /// assert_eq!(Duration { sec: -12, nsec: -123 }.nanos(), -12_000_000_123);
    /// ```
    #[inline]
    pub fn nanos(self) -> i64 {
        i64::from(self.sec) * BILLION + i64::from(self.nsec)
    }

    /// Returns the number of seconds in the duration.
    ///
    /// # Examples
    ///
    /// ```
    /// # use ros_message::Duration;
    /// assert_eq!(Duration { sec: 0, nsec: 0 }.seconds(), 0.0);
    /// assert_eq!(Duration { sec: 12, nsec: 123 }.seconds(), 12.000_000_123);
    /// assert_eq!(Duration { sec: -12, nsec: -123 }.seconds(), -12.000_000_123);
    /// ```
    #[inline]
    pub fn seconds(self) -> f64 {
        f64::from(self.sec) + f64::from(self.nsec) / BILLION as f64
    }
}

impl cmp::PartialEq for Duration {
    fn eq(&self, other: &Self) -> bool {
        self.nanos() == other.nanos()
    }
}

impl cmp::PartialOrd for Duration {
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        self.nanos().partial_cmp(&other.nanos())
    }
}

impl cmp::Ord for Duration {
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        self.nanos().cmp(&other.nanos())
    }
}

impl ops::Add<Duration> for Time {
    type Output = Time;
    fn add(self, rhs: Duration) -> Self::Output {
        Time::from_nanos(self.nanos() + rhs.nanos())
    }
}

impl ops::Add<Duration> for Duration {
    type Output = Duration;
    fn add(self, rhs: Duration) -> Self::Output {
        Duration::from_nanos(self.nanos() + rhs.nanos())
    }
}

impl ops::Sub<Time> for Time {
    type Output = Duration;
    fn sub(self, rhs: Time) -> Self::Output {
        Duration::from_nanos(self.nanos() - rhs.nanos())
    }
}

impl ops::Sub<Duration> for Time {
    type Output = Time;
    fn sub(self, rhs: Duration) -> Self::Output {
        Time::from_nanos(self.nanos() - rhs.nanos())
    }
}

impl ops::Sub<Duration> for Duration {
    type Output = Duration;
    fn sub(self, rhs: Duration) -> Self::Output {
        Duration::from_nanos(self.nanos() - rhs.nanos())
    }
}

impl ops::Neg for Duration {
    type Output = Duration;
    fn neg(self) -> Self::Output {
        Duration {
            sec: -self.sec,
            nsec: -self.nsec,
        }
    }
}

impl From<time::Duration> for Duration {
    fn from(std_duration: time::Duration) -> Self {
        Duration {
            sec: std_duration.as_secs() as i32,
            nsec: std_duration.subsec_nanos() as i32,
        }
    }
}
