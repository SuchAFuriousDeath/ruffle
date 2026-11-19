//! Temporary timing instrumentation for CI performance experiments.

use std::time::{Instant, SystemTime, UNIX_EPOCH};

fn now_ms() -> f64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs_f64() * 1000.0)
        .unwrap_or_default()
}

pub fn log(label: &str, start: Instant) {
    eprintln!(
        "[TIMING] t={:.3} pid={} {} {:.3}ms",
        now_ms(),
        std::process::id(),
        label,
        start.elapsed().as_secs_f64() * 1000.0
    );
}

pub fn mark(label: &str) {
    eprintln!(
        "[TIMING] t={:.3} pid={} {} 0ms",
        now_ms(),
        std::process::id(),
        label
    );
}

/// Logs the elapsed time when dropped.
pub struct Timer {
    label: String,
    start: Instant,
}

impl Timer {
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            start: Instant::now(),
        }
    }
}

impl Drop for Timer {
    fn drop(&mut self) {
        log(&self.label, self.start);
    }
}
