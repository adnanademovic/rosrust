use std::ops;

const BILLION: i64 = 1000000000;

#[derive(Serialize, Deserialize, Debug)]
pub struct Time {
    pub sec: i32,
    pub nsec: i32,
}

impl Time {
    pub fn new() -> Time {
        Time { sec: 0, nsec: 0 }
    }

    pub fn from_nanos(t: i64) -> Time {
        Time {
            sec: (t / BILLION) as i32,
            nsec: (t % BILLION) as i32,
        }
    }

    fn nanos(self) -> i64 {
        self.sec as i64 * BILLION + self.nsec as i64
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Duration {
    pub sec: i32,
    pub nsec: i32,
}

impl Duration {
    pub fn new() -> Duration {
        Duration { sec: 0, nsec: 0 }
    }

    pub fn from_nanos(t: i64) -> Duration {
        Duration {
            sec: (t / BILLION) as i32,
            nsec: (t % BILLION) as i32,
        }
    }

    fn nanos(self) -> i64 {
        self.sec as i64 * BILLION + self.nsec as i64
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

#[cfg(test)]
mod tests {
    use super::{Duration, Time};

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
    fn works_with_negative() {
        let time = Time::from_nanos(-123456789987654321);
        assert_eq!(time.sec, -123456789);
        assert_eq!(time.nsec, -987654321);
        assert_eq!(time.nanos(), -123456789987654321);
        let time = Duration::from_nanos(-123456789987654321);
        assert_eq!(time.sec, -123456789);
        assert_eq!(time.nsec, -987654321);
        assert_eq!(time.nanos(), -123456789987654321);
    }
}
