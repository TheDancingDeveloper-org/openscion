//! The engine's clock port.
//!
//! A changeset's `created_at` is assigned by the server and is part of its id,
//! so the engine reads time through [`Clock`] rather than calling the system
//! clock directly. Tests and benchmarks pass a [`FixedClock`] and get the same
//! ids on every run.

use std::fmt;
use std::time::{SystemTime, UNIX_EPOCH};

/// A source of the current time.
pub trait Clock: Send + Sync + fmt::Debug {
    /// Milliseconds since the Unix epoch.
    fn now_millis(&self) -> u64;
}

/// The system clock. A time before the epoch reads as 0.
#[derive(Debug, Clone, Copy, Default)]
pub struct SystemClock;

impl Clock for SystemClock {
    fn now_millis(&self) -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_or(0, |elapsed| {
                u64::try_from(elapsed.as_millis()).unwrap_or(u64::MAX)
            })
    }
}

/// A clock that always reads the same time.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FixedClock(pub u64);

impl Clock for FixedClock {
    fn now_millis(&self) -> u64 {
        self.0
    }
}

#[cfg(test)]
mod test;
