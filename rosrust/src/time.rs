use serde_derive::{Deserialize, Serialize};
use std::cmp;
use std::ops;
use std::time;

const BILLION: i64 = 1_000_000_000;

#[derive(Copy, Clone, Default, Serialize, Deserialize, Debug)]
pub struct Time {
    pub sec: u32,
    pub nsec: u32,
}

impl Time {
    #[inline]
    pub fn new() -> Time {
        Self::default()
    }

    #[inline]
    pub fn from_nanos(t: i64) -> Time {
        Time {
            sec: (t / BILLION) as u32,
            nsec: (t % BILLION) as u32,
        }
    }

    #[inline]
    pub fn nanos(self) -> i64 {
        i64::from(self.sec) * BILLION + i64::from(self.nsec)
    }

    #[inline]
    pub fn seconds(self) -> f64 {
        f64::from(self.sec) + f64::from(self.nsec) / BILLION as f64
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

impl cmp::Eq for Time {}

impl cmp::Ord for Time {
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        self.nanos().cmp(&other.nanos())
    }
}

#[derive(Copy, Clone, Default, Serialize, Deserialize, Debug)]
pub struct Duration {
    pub sec: i32,
    pub nsec: i32,
}

impl Duration {
    #[inline]
    pub fn new() -> Duration {
        Self::default()
    }

    #[inline]
    pub fn from_nanos(t: i64) -> Duration {
        Duration {
            sec: (t / BILLION) as i32,
            nsec: (t % BILLION) as i32,
        }
    }

    #[inline]
    pub fn from_seconds(sec: i32) -> Duration {
        Duration { sec, nsec: 0 }
    }

    #[inline]
    fn nanos(self) -> i64 {
        i64::from(self.sec) * BILLION + i64::from(self.nsec)
    }

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

impl cmp::Eq for Duration {}

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

#[cfg(test)]
mod tests {
    use super::{Duration, Time};
    use std::time;

    #[test]
    fn from_nanos_works() {
        let time = Time::from_nanos(123456789987654321);
        assert_eq!(time.sec, 123456789);
        assert_eq!(time.nsec, 987654321);
        let time = Duration::from_nanos(123456789987654321);
        assert_eq!(time.sec, 123456789);
        assert_eq!(time.nsec, 987654321);
    }

    #[test]
    fn nanos_works() {
        let time = Time {
            sec: 123456789,
            nsec: 987654321,
        };
        assert_eq!(time.nanos(), 123456789987654321);
        let time = Duration {
            sec: 123456789,
            nsec: 987654321,
        };
        assert_eq!(time.nanos(), 123456789987654321);
    }

    #[test]
    fn duration_works_with_negative() {
        let time = Duration::from_nanos(-123456789987654321);
        assert_eq!(time.sec, -123456789);
        assert_eq!(time.nsec, -987654321);
        assert_eq!(time.nanos(), -123456789987654321);
    }

    #[test]
    fn convert_works() {
        let std_duration = time::Duration::new(123, 456);
        let msg_duration = Duration::from(std_duration);
        assert_eq!(msg_duration.sec, 123);
        assert_eq!(msg_duration.nsec, 456);

        let std_duration2 = time::Duration::new(9876, 54321);
        let msg_duration2: Duration = std_duration2.into();
        assert_eq!(msg_duration2.sec, 9876);
        assert_eq!(msg_duration2.nsec, 54321);
    }
}
