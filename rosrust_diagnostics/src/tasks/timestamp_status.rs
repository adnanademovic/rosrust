use crate::{Level, Status, Task};
use rosrust::Time;
use std::sync::Mutex;

pub struct TimestampStatusBuilder<'a> {
    min_acceptable: f64,
    max_acceptable: f64,
    name: &'a str,
}

impl<'a> TimestampStatusBuilder<'a> {
    #[inline]
    fn new() -> Self {
        Self {
            min_acceptable: -1.0,
            max_acceptable: 5.0,
            name: "Timestamp Status",
        }
    }

    #[inline]
    pub fn min_acceptable(&mut self, value: f64) -> &mut Self {
        self.min_acceptable = value;
        self
    }

    #[inline]
    pub fn max_acceptable(&mut self, value: f64) -> &mut Self {
        self.max_acceptable = value;
        self
    }

    #[inline]
    pub fn name(&mut self, name: &'a str) -> &mut Self {
        self.name = name;
        self
    }

    #[inline]
    pub fn build(&self) -> TimestampStatus {
        TimestampStatus::new(self.min_acceptable, self.max_acceptable, self.name.into())
    }
}

pub struct TimestampStatus {
    acceptable: Range,
    name: String,
    tracker: Mutex<Tracker>,
}

impl TimestampStatus {
    #[inline]
    pub fn builder<'a>() -> TimestampStatusBuilder<'a> {
        TimestampStatusBuilder::new()
    }

    #[inline]
    pub fn new(min_acceptable: f64, max_acceptable: f64, name: String) -> Self {
        Self {
            acceptable: Range::range(min_acceptable, max_acceptable),
            name,
            tracker: Mutex::new(Tracker::default()),
        }
    }

    pub fn tick_float(&self, timestamp: f64) {
        let mut tracker = self.tracker.lock().expect(FAILED_TO_LOCK);

        if timestamp == 0.0 {
            tracker.zero_seen = true;
            return;
        }

        let delta = rosrust::now().seconds() - timestamp;

        if tracker.delta_valid {
            tracker.delta_range.combine_with(delta);
        } else {
            tracker.delta_valid = true;
            tracker.delta_range = Range::new(delta);
        }
    }

    #[inline]
    pub fn tick(&self, timestamp: &Time) {
        self.tick_float(timestamp.seconds())
    }
}

impl Task for TimestampStatus {
    #[inline]
    fn name(&self) -> &str {
        &self.name
    }

    fn run(&self, status: &mut Status) {
        let mut tracker = self.tracker.lock().expect(FAILED_TO_LOCK);

        status.set_summary(Level::Ok, "Timestamps are reasonable.");

        if !tracker.delta_valid {
            status.set_summary(Level::Warn, "No data since last update.");
        } else {
            if tracker.delta_range.min < self.acceptable.min {
                status.set_summary(Level::Error, "Timestamps too far in future seen.");
                tracker.counts.early += 1;
            }

            if tracker.delta_range.max > self.acceptable.max {
                status.set_summary(Level::Error, "Timestamps too far in past seen.");
                tracker.counts.late += 1;
            }

            if tracker.zero_seen {
                status.set_summary(Level::Error, "Zero timestamp seen.");
                tracker.counts.zero += 1;
            }
        }

        status.add("Earliest timestamp delay:", tracker.delta_range.min);
        status.add("Latest timestamp delay:", tracker.delta_range.max);

        status.add("Earliest acceptable timestamp delay:", self.acceptable.min);
        status.add("Latest acceptable timestamp delay:", self.acceptable.max);

        status.add("Late diagnostic update count:", tracker.counts.late);
        status.add("Early diagnostic update count:", tracker.counts.early);
        status.add("Zero seen diagnostic update count:", tracker.counts.zero);

        tracker.clear_last_reading();
    }
}

#[derive(Default)]
struct Tracker {
    counts: Counter,
    zero_seen: bool,
    delta_range: Range,
    delta_valid: bool,
}

impl Tracker {
    fn clear_last_reading(&mut self) {
        self.zero_seen = false;
        self.delta_range = Range::default();
        self.delta_valid = false;
    }
}

#[derive(Default)]
struct Counter {
    early: usize,
    late: usize,
    zero: usize,
}

struct Range {
    min: f64,
    max: f64,
}

impl Default for Range {
    #[inline]
    fn default() -> Self {
        Self::new(0.0)
    }
}

impl Range {
    #[inline]
    fn new(value: f64) -> Self {
        Self::range(value, value)
    }

    #[inline]
    fn range(min: f64, max: f64) -> Self {
        Self { min, max }
    }

    fn combine_with(&mut self, value: f64) {
        self.min = self.min.min(value);
        self.max = self.max.max(value);
    }
}

static FAILED_TO_LOCK: &'static str = "Failed to acquire lock";
