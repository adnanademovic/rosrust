use crate::util::FAILED_TO_LOCK;
use crossbeam::sync::{Parker, Unparker};
use ros_message::{Duration, Time};
use std::cell::Cell;
use std::cmp;
use std::collections::BinaryHeap;
use std::sync::{Arc, Mutex};
use std::thread::sleep;
use std::time::{Duration as StdDuration, SystemTime, UNIX_EPOCH};

static BEFORE_EPOCH: &str = "Requested time is before UNIX epoch.";

pub struct Delay {
    clock: Arc<dyn Clock>,
    delay: Duration,
}

impl Delay {
    pub fn new(clock: Arc<dyn Clock>, delay: Duration) -> Self {
        Self { clock, delay }
    }

    pub fn sleep(self) {
        self.clock.sleep(self.delay);
    }
}

pub struct Rate {
    clock: Arc<dyn Clock>,
    next: Cell<Time>,
    delay: Duration,
}

impl Rate {
    pub fn new(clock: Arc<dyn Clock>, delay: Duration) -> Rate {
        let start = clock.now();
        Rate {
            clock,
            next: Cell::new(start),
            delay,
        }
    }

    pub fn sleep(&self) {
        let new_time = self.next.get() + self.delay;
        self.next.set(new_time);
        self.clock.wait_until(new_time);
    }
}

pub trait Clock: Send + Sync {
    fn now(&self) -> Time;
    fn sleep(&self, d: Duration);
    fn wait_until(&self, t: Time);
    fn await_init(&self) {}
}

#[derive(Clone, Default)]
pub struct RealClock {}

impl Clock for RealClock {
    #[inline]
    fn now(&self) -> Time {
        let time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect(BEFORE_EPOCH);
        Time {
            sec: time.as_secs() as u32,
            nsec: time.subsec_nanos(),
        }
    }

    #[inline]
    fn sleep(&self, d: Duration) {
        if d < Duration::default() {
            return;
        }
        sleep(StdDuration::new(d.sec as u64, d.nsec as u32));
    }

    #[inline]
    fn wait_until(&self, t: Time) {
        self.sleep(t - self.now());
    }
}

struct Timeout {
    timestamp: Time,
    unparker: Unparker,
}

impl Drop for Timeout {
    fn drop(&mut self) {
        self.unparker.unpark();
    }
}

impl cmp::PartialEq for Timeout {
    fn eq(&self, other: &Self) -> bool {
        self.timestamp == other.timestamp
    }
}

impl cmp::PartialOrd for Timeout {
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        self.timestamp
            .partial_cmp(&other.timestamp)
            .map(cmp::Ordering::reverse)
    }
}

impl cmp::Eq for Timeout {}

impl cmp::Ord for Timeout {
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        self.timestamp.cmp(&other.timestamp).reverse()
    }
}

#[derive(Default)]
pub struct SimData {
    current: Time,
    timeouts: BinaryHeap<Timeout>,
}

#[derive(Default)]
pub struct SimulatedClock {
    pub data: Mutex<SimData>,
}

impl SimulatedClock {
    pub fn trigger(&self, time: Time) {
        let mut data = self.data.lock().expect(FAILED_TO_LOCK);
        data.current = time;
        loop {
            match data.timeouts.peek() {
                None => break,
                Some(next) if next.timestamp > data.current => break,
                _ => {}
            }
            data.timeouts.pop();
        }
    }
}

impl Clock for SimulatedClock {
    #[inline]
    fn now(&self) -> Time {
        self.data.lock().expect(FAILED_TO_LOCK).current
    }

    #[inline]
    fn sleep(&self, d: Duration) {
        if d.sec < 0 || d.nsec < 0 {
            return;
        }
        let current = { self.data.lock().expect(FAILED_TO_LOCK).current };
        self.wait_until(current + d);
    }

    #[inline]
    fn wait_until(&self, timestamp: Time) {
        let parker = Parker::new();
        let unparker = parker.unparker().clone();
        {
            self.data
                .lock()
                .expect(FAILED_TO_LOCK)
                .timeouts
                .push(Timeout {
                    timestamp,
                    unparker,
                });
        }
        parker.park()
    }

    fn await_init(&self) {
        if self.data.lock().expect(FAILED_TO_LOCK).current == Time::default() {
            self.wait_until(Time::from_nanos(1));
        }
    }
}
