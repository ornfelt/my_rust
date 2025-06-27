use std::time::{Duration, Instant};

/// A simple stopwatch, mirroring the Java logic.
pub struct Stopwatch {
    start_time:   Option<Instant>,
    elapsed:      Duration,
}

impl Stopwatch {
    /// Creates a new, stopped stopwatch with zero elapsed time.
    pub fn new() -> Self {
        Self {
            start_time: None,
            elapsed:    Duration::ZERO,
        }
    }

    /// Starts timing. If already running, does nothing.
    pub fn start(&mut self) {
        if self.start_time.is_none() {
            self.start_time = Some(Instant::now());
        }
    }

    /// Stops timing and accumulates the time since the last start.
    /// If already stopped, does nothing.
    pub fn stop(&mut self) {
        if let Some(since) = self.start_time.take() {
            self.elapsed += since.elapsed();
        }
    }

    /// Stops and clears the accumulated time to zero.
    pub fn reset(&mut self) {
        self.elapsed = Duration::ZERO;
        self.start_time = None;
    }

    /// Resets to zero and starts immediately.
    pub fn restart(&mut self) {
        self.elapsed = Duration::ZERO;
        self.start_time = Some(Instant::now());
    }

    /// Returns true if the stopwatch is currently running.
    pub fn is_running(&self) -> bool {
        self.start_time.is_some()
    }

    /// Returns the elapsed time in nanoseconds, including current segment if running.
    pub fn elapsed_nanos(&self) -> u128 {
        let extra = self
            .start_time
            .map(|s| s.elapsed())
            .unwrap_or_else(|| Duration::ZERO);
        (self.elapsed + extra).as_nanos()
    }

    /// Returns the elapsed time in milliseconds.
    pub fn elapsed_millis(&self) -> u128 {
        self.elapsed_nanos() / 1_000_000
    }

    /// Returns the elapsed time in seconds (fractional).
    pub fn elapsed_seconds(&self) -> f64 {
        self.elapsed_nanos() as f64 / 1_000_000_000.0
    }

    /// Alias for `elapsed_millis()`
    pub fn time(&self) -> u128 {
        self.elapsed_millis()
    }
}
