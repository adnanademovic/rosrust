use serde_derive::{Deserialize, Serialize};
use std::cmp;
use std::fmt;
use std::fmt::Formatter;
use std::hash::{Hash, Hasher};
use std::ops;
use std::time;

const BILLION: i64 = 1_000_000_000;

#[derive(Copy, Clone, Default, Serialize, Deserialize, Debug, Eq)]
pub struct Time {
    pub sec: u32,
    pub nsec: u32,
}

impl Hash for Time {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.nanos().hash(state)
    }
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

fn display_nanos(nanos: &str, f: &mut Formatter<'_>) -> fmt::Result {
    let split_point = nanos.len() - 9;
    let characters = nanos.chars();
    let (left, right) = characters.as_str().split_at(split_point);
    let right = right.trim_end_matches('0');
    if right.is_empty() {
        write!(f, "{}", left)
    } else {
        write!(f, "{}.{}", left, right)
    }
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

#[derive(Copy, Clone, Default, Serialize, Deserialize, Debug, Eq)]
pub struct Duration {
    pub sec: i32,
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
    pub fn nanos(self) -> i64 {
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
