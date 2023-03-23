#![allow(clippy::float_cmp)]

use crate::{Level, Status, Task};
use rosrust::Time;
use std::sync::Mutex;

/// The structure for building a timestamp status task.
///
/// Use `TimestampStatus::builder()` to create an instance of this structure.
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

    /// Sets the minimum difference in timestamp that is expected, in seconds.
    ///
    /// Defaults to `-1.0`, which should be impossible to trigger if correctly used.
    #[inline]
    pub fn min_acceptable(&mut self, value: f64) -> &mut Self {
        self.min_acceptable = value;
        self
    }

    /// Sets the maximum difference in timestamp that is expected, in seconds.
    ///
    /// Defaults to `5` seconds.
    #[inline]
    pub fn max_acceptable(&mut self, value: f64) -> &mut Self {
        self.max_acceptable = value;
        self
    }

    /// Sets the name of the task.
    ///
    /// Defaults to "Timestamp Status".
    #[inline]
    pub fn name(&mut self, name: &'a str) -> &mut Self {
        self.name = name;
        self
    }

    /// Builds the timestamp status with the provided parameters.
    #[inline]
    pub fn build(&self) -> TimestampStatus {
        TimestampStatus::new(self.min_acceptable, self.max_acceptable, self.name.into())
    }
}

/// Diagnostic task to monitor the interval between events.
///
/// This diagnostic task monitors the difference between consecutive events,
/// and creates corresponding diagnostics. An error occurs if the interval
/// between consecutive events is too large or too small. An error condition
/// will only be reported during a single diagnostic report unless it
/// persists. Tallies of errors are also maintained to keep track of errors
/// in a more persistent way.
pub struct TimestampStatus {
    acceptable: Range,
    name: String,
    tracker: Mutex<Tracker>,
}

impl TimestampStatus {
    /// Creates a builder for a new timestamp status task.
    #[inline]
    pub fn builder<'a>() -> TimestampStatusBuilder<'a> {
        TimestampStatusBuilder::new()
    }

    /// Creates a new timestamp status based on the provided parameters.
    ///
    /// Look at the `TimestampStatusBuilder` for more information about the parameters and
    /// reasonable defaults.
    #[inline]
    pub fn new(min_acceptable: f64, max_acceptable: f64, name: String) -> Self {
        Self {
            acceptable: Range::range(min_acceptable, max_acceptable),
            name,
            tracker: Mutex::new(Tracker::default()),
        }
    }

    /// Signals an event, with the timestamp provided as a float point in seconds.
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

    /// Signals an event.
    #[inline]
    pub fn tick(&self, timestamp: Time) {
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

    #[allow(clippy::self_named_constructors)]
    #[inline]
    fn range(min: f64, max: f64) -> Self {
        Self { min, max }
    }

    fn combine_with(&mut self, value: f64) {
        self.min = self.min.min(value);
        self.max = self.max.max(value);
    }
}

static FAILED_TO_LOCK: &str = "Failed to acquire lock";

#[cfg(test)]
mod tests {
    use super::*;
    use quickcheck::{Arbitrary, Gen};
    use quickcheck_macros::quickcheck;

    #[quickcheck]
    fn timestamp_status_constructor_initializes_properly(low: f64, high: f64, name: String) {
        let ts = TimestampStatus::new(low, high, name.clone());
        let tracker = ts.tracker.lock().unwrap();

        assert_eq!(ts.acceptable.min, low);
        assert_eq!(ts.acceptable.max, high);
        assert_eq!(ts.name, name);
        assert!(!tracker.zero_seen);
        assert!(!tracker.delta_valid);
        assert_eq!(tracker.counts.early, 0);
        assert_eq!(tracker.counts.late, 0);
        assert_eq!(tracker.counts.zero, 0);
        assert_eq!(tracker.delta_range.min, 0.0);
        assert_eq!(tracker.delta_range.max, 0.0);
    }

    #[quickcheck]
    fn timestamp_status_builder_initializes_properly(
        low: Option<f64>,
        high: Option<f64>,
        name: Option<String>,
    ) {
        let mut tsb = TimestampStatus::builder();
        if let Some(low) = low {
            tsb.min_acceptable(low);
        }
        if let Some(high) = high {
            tsb.max_acceptable(high);
        }
        if let Some(ref name) = name {
            tsb.name(name);
        }
        let ts = tsb.build();

        let low = low.unwrap_or(-1.0);
        let high = high.unwrap_or(5.0);
        let name = name.unwrap_or_else(|| "Timestamp Status".into());
        let tracker = ts.tracker.lock().unwrap();

        assert_eq!(ts.acceptable.min, low);
        assert_eq!(ts.acceptable.max, high);
        assert_eq!(ts.name, name);
        assert!(!tracker.zero_seen);
        assert!(!tracker.delta_valid);
        assert_eq!(tracker.counts.early, 0);
        assert_eq!(tracker.counts.late, 0);
        assert_eq!(tracker.counts.zero, 0);
        assert_eq!(tracker.delta_range.min, 0.0);
        assert_eq!(tracker.delta_range.max, 0.0);
    }

    #[test]
    fn counter_defaults_to_zeros() {
        let counter = Counter::default();
        assert_eq!(counter.early, 0);
        assert_eq!(counter.late, 0);
        assert_eq!(counter.zero, 0);
    }

    #[test]
    fn range_defaults_to_zeros() {
        let range = Range::default();
        assert_eq!(range.min, 0.0);
        assert_eq!(range.max, 0.0);
    }

    #[quickcheck]
    fn range_new_sets_both_values_to_input(value: f64) {
        let range = Range::new(value);
        assert_eq!(range.min, value);
        assert_eq!(range.max, value);
    }

    #[quickcheck]
    fn range_range_sets_respective_values_to_inputs(min: f64, max: f64) {
        let range = Range::range(min, max);
        assert_eq!(range.min, min);
        assert_eq!(range.max, max);
    }

    #[derive(Clone, Debug)]
    struct SortedTriplet {
        low: f64,
        mid: f64,
        high: f64,
    }

    impl From<(f64, f64, f64)> for SortedTriplet {
        fn from((a, b, c): (f64, f64, f64)) -> Self {
            let mut inputs = [a, b, c];
            inputs.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Less));
            let [low, mid, high] = inputs;
            SortedTriplet { low, mid, high }
        }
    }

    impl Arbitrary for SortedTriplet {
        fn arbitrary<G: Gen>(g: &mut G) -> Self {
            <(f64, f64, f64)>::arbitrary(g).into()
        }

        fn shrink(&self) -> Box<dyn Iterator<Item = Self>> {
            Box::new(Arbitrary::shrink(&(self.low, self.mid, self.high)).map(Into::into))
        }
    }

    #[quickcheck]
    fn range_combination_with_higher_increases_max(triplet: SortedTriplet) {
        let mut range = Range::range(triplet.low, triplet.mid);
        range.combine_with(triplet.high);
        assert_eq!(range.min, triplet.low);
        assert_eq!(range.max, triplet.high);
    }

    #[quickcheck]
    fn range_combination_with_lower_decreases_min(triplet: SortedTriplet) {
        let mut range = Range::range(triplet.mid, triplet.high);
        range.combine_with(triplet.low);
        assert_eq!(range.min, triplet.low);
        assert_eq!(range.max, triplet.high);
    }

    #[quickcheck]
    fn range_combination_with_inbetween_does_nothing(triplet: SortedTriplet) {
        let mut range = Range::range(triplet.low, triplet.high);
        range.combine_with(triplet.mid);
        assert_eq!(range.min, triplet.low);
        assert_eq!(range.max, triplet.high);
    }

    #[quickcheck]
    fn range_combination_fixes_reverse_range(triplet: SortedTriplet) {
        let mut range = Range::range(triplet.high, triplet.low);
        range.combine_with(triplet.mid);
        assert_eq!(range.min, triplet.mid);
        assert_eq!(range.max, triplet.mid);
    }

    #[test]
    fn tracker_defaults_to_zeros_and_false() {
        #[allow(clippy::field_reassign_with_default)]
        let tracker = Tracker::default();
        assert!(!tracker.zero_seen);
        assert!(!tracker.delta_valid);
        assert_eq!(tracker.counts.early, 0);
        assert_eq!(tracker.counts.late, 0);
        assert_eq!(tracker.counts.zero, 0);
        assert_eq!(tracker.delta_range.min, 0.0);
        assert_eq!(tracker.delta_range.max, 0.0);
    }

    #[test]
    fn tracker_clearing_reading_only_maintains_counts() {
        #![allow(clippy::field_reassign_with_default)]
        let mut tracker = Tracker::default();
        tracker.zero_seen = true;
        tracker.delta_valid = true;
        tracker.counts.early = 5;
        tracker.counts.late = 6;
        tracker.counts.zero = 7;
        tracker.delta_range.min = -5.0;
        tracker.delta_range.max = 14.0;

        tracker.clear_last_reading();

        assert!(!tracker.zero_seen);
        assert!(!tracker.delta_valid);
        assert_eq!(tracker.counts.early, 5);
        assert_eq!(tracker.counts.late, 6);
        assert_eq!(tracker.counts.zero, 7);
        assert_eq!(tracker.delta_range.min, 0.0);
        assert_eq!(tracker.delta_range.max, 0.0);
    }
}
