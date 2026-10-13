//! Unit tests for the clock implementations.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use super::{Clock, FixedClock, SystemClock};

#[test]
fn a_fixed_clock_reads_its_value() {
    assert_eq!(FixedClock(42).now_millis(), 42);
}

#[test]
fn the_system_clock_reads_after_2020() {
    // 2020-01-01T00:00:00Z in milliseconds.
    assert!(SystemClock.now_millis() > 1_577_836_800_000);
}
