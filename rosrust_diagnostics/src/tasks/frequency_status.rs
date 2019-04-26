use crate::{Level, Status, Task};
use rosrust::Time;
use std::collections::VecDeque;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

pub struct FrequencyStatusBuilder<'a> {
    min_frequency: f64,
    max_frequency: f64,
    tolerance: f64,
    window_size: usize,
    ticker: Option<&'a FrequencyStatusTicker>,
    name: &'a str,
}

impl<'a> FrequencyStatusBuilder<'a> {
    #[inline]
    fn new() -> Self {
        Self {
            min_frequency: 0.0,
            max_frequency: std::f64::INFINITY,
            tolerance: 0.1,
            window_size: 5,
            ticker: None,
            name: "Frequency Status",
        }
    }

    #[inline]
    pub fn min_frequency(&mut self, value: f64) -> &mut Self {
        self.min_frequency = value;
        self
    }

    #[inline]
    pub fn max_frequency(&mut self, value: f64) -> &mut Self {
        self.max_frequency = value;
        self
    }

    #[inline]
    pub fn tolerance(&mut self, value: f64) -> &mut Self {
        self.tolerance = value;
        self
    }

    #[inline]
    pub fn window_size(&mut self, value: usize) -> &mut Self {
        self.window_size = value;
        self
    }

    #[inline]
    pub fn name(&mut self, name: &'a str) -> &mut Self {
        self.name = name;
        self
    }

    #[inline]
    pub fn ticker(&mut self, ticker: &'a FrequencyStatusTicker) -> &mut Self {
        self.ticker = Some(ticker);
        self
    }

    #[inline]
    pub fn build(&self) -> FrequencyStatus {
        FrequencyStatus::new(
            self.min_frequency,
            self.max_frequency,
            self.tolerance,
            self.window_size,
            self.ticker
                .cloned()
                .unwrap_or_else(FrequencyStatusTicker::new),
            self.name.into(),
        )
    }
}

#[derive(Clone)]
pub struct FrequencyStatusTicker {
    tick_count: Arc<AtomicUsize>,
}

impl FrequencyStatusTicker {
    #[inline]
    pub fn new() -> Self {
        Self {
            tick_count: Arc::new(AtomicUsize::new(0)),
        }
    }

    #[inline]
    pub fn tick(&self) {
        self.tick_count.fetch_add(1, Ordering::SeqCst);
    }

    #[inline]
    fn read(&self) -> usize {
        self.tick_count.load(Ordering::SeqCst)
    }
}

pub struct FrequencyStatus {
    min_frequency: f64,
    max_frequency: f64,
    min_tolerated_frequency: f64,
    max_tolerated_frequency: f64,
    name: String,
    ticker: FrequencyStatusTicker,
    history: Arc<Mutex<VecDeque<HistoryEntry>>>,
}

impl FrequencyStatus {
    #[inline]
    pub fn builder<'a>() -> FrequencyStatusBuilder<'a> {
        FrequencyStatusBuilder::new()
    }

    fn new(
        min_frequency: f64,
        max_frequency: f64,
        tolerance: f64,
        window_size: usize,
        ticker: FrequencyStatusTicker,
        name: String,
    ) -> Self {
        let history_entry = HistoryEntry::new(0);

        let mut history = VecDeque::with_capacity(window_size);
        history.extend((0..window_size).map(|_| history_entry.clone()));

        Self {
            min_frequency,
            max_frequency,
            min_tolerated_frequency: min_frequency * (1.0 - tolerance),
            max_tolerated_frequency: max_frequency * (1.0 + tolerance),
            name,
            ticker,
            history: Arc::new(Mutex::new(history)),
        }
    }

    #[inline]
    pub fn create_ticker() -> FrequencyStatusTicker {
        FrequencyStatusTicker::new()
    }

    #[inline]
    pub fn ticker(&self) -> &FrequencyStatusTicker {
        &self.ticker
    }

    fn frequency_to_summary(&self, frequency: f64) -> (Level, &str) {
        match frequency {
            v if v == 0.0 => (Level::Error, "No events recorded."),
            v if v < self.min_tolerated_frequency => (Level::Warn, "Frequency too low."),
            v if v > self.max_tolerated_frequency => (Level::Warn, "Frequency too high."),
            _ => (Level::Ok, "Desired frequency met"),
        }
    }

    fn add_frequency_info(&self, status: &mut Status) -> () {
        if self.max_frequency == self.min_frequency {
            status.add("Target frequency (Hz)", self.min_frequency)
        }
        if self.min_frequency > 0.0 {
            status.add(
                "Minimum acceptable frequency (Hz)",
                self.min_tolerated_frequency,
            )
        }
        if self.max_frequency != std::f64::INFINITY {
            status.add(
                "Maximum acceptable frequency (Hz)",
                self.max_tolerated_frequency,
            )
        }
    }
}

impl Task for FrequencyStatus {
    #[inline]
    fn name(&self) -> &str {
        &self.name
    }

    fn run(&self, status: &mut Status) {
        let mut history = match self.history.lock() {
            Ok(value) => value,
            Err(_err) => {
                status.set_summary(
                    Level::Error,
                    "Failed to acquire Mutex lock inside frequency check. This can only be caused by a thread unexpectedly crashing inside the node.",
                );
                return;
            }
        };
        let history_end = HistoryEntry::new(self.ticker.read());

        let end_count = history_end.count.clone();
        let end_time = history_end.time.clone();

        let history_start = match history.pop_front() {
            Some(value) => value,
            None => {
                status.set_summary(
                    Level::Error,
                    "History in frequency status tracker is unexpectedly missing elements.",
                );
                return;
            }
        };
        history.push_back(history_end);

        let events = end_count - history_start.count;
        let window = (end_time - history_start.time).seconds();
        let frequency = events as f64 / window;

        let (level, message) = self.frequency_to_summary(frequency);
        status.set_summary(level, message);

        status.add("Events in window", events);
        status.add("Events since startup", end_count);
        status.add("Duration of window (s)", window);
        status.add("Actual frequency (Hz)", frequency);

        self.add_frequency_info(status)
    }
}

#[derive(Clone)]
struct HistoryEntry {
    count: usize,
    time: Time,
}

impl HistoryEntry {
    fn new(count: usize) -> HistoryEntry {
        HistoryEntry {
            count,
            time: rosrust::now(),
        }
    }
}
