use std::cmp;
use std::collections::BinaryHeap;
use std::sync::{Arc, Mutex};
use std::sync::mpsc::{channel, Sender};
use std::thread::sleep;
use std::time::{Duration as StdDuration, SystemTime, UNIX_EPOCH};
use time::{Duration, Time};

static BEFORE_EPOCH: &'static str = "Requested time is before UNIX epoch.";

pub struct Rate {
    clock: Arc<Clock>,
    next: Time,
    delay: Duration,
}

impl Rate {
    pub fn new(clock: Arc<Clock>, delay: Duration) -> Rate {
        let start = clock.now();
        Rate {
            clock,
            next: start,
            delay,
        }
    }

    pub fn sleep(&mut self) {
        self.next = self.next.clone() + self.delay.clone();
        self.clock.wait_until(self.next.clone());
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
            sec: time.as_secs() as i32,
            nsec: time.subsec_nanos() as i32,
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
    tx: Sender<()>,
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
            if let Some(next) = data.timeouts.pop() {
                next.tx.send(()).expect(SLEEPING_THREAD_MISSING);
            }
        }
    }
}

impl Clock for SimulatedClock {
    #[inline]
    fn now(&self) -> Time {
        self.data.lock().expect(FAILED_TO_LOCK).current.clone()
    }

    #[inline]
    fn sleep(&self, d: Duration) {
        if d.sec < 0 || d.nsec < 0 {
            return;
        }
        let current = { self.data.lock().expect(FAILED_TO_LOCK).current.clone() };
        self.wait_until(current + d);
    }

    #[inline]
    fn wait_until(&self, timestamp: Time) {
        let (tx, rx) = channel();
        {
            self.data
                .lock()
                .expect(FAILED_TO_LOCK)
                .timeouts
                .push(Timeout { timestamp, tx });
        }
        if rx.recv().is_err() {
            warn!("Sleep beyond simulated clock");
        }
    }

    fn await_init(&self) {
        if self.data.lock().expect(FAILED_TO_LOCK).current == Time::default() {
            self.wait_until(Time::from_nanos(1));
        }
    }
}

static FAILED_TO_LOCK: &'static str = "Failed to acquire lock";
static SLEEPING_THREAD_MISSING: &'static str = "Failed to find sleeping thread";
