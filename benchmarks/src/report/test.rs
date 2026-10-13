//! Unit tests for medians and Markdown rendering.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::time::Duration;

use super::{Measurement, Report};
use crate::{Error, Scenario};

fn measurement(runs: &[u64]) -> Measurement {
    Measurement {
        backend: "git",
        scenario: Scenario::Import,
        runs: runs.iter().copied().map(Duration::from_millis).collect(),
    }
}

#[test]
fn the_median_of_an_odd_count_is_the_middle_run() {
    assert_eq!(
        measurement(&[9, 1, 5]).median().unwrap(),
        Duration::from_millis(5)
    );
}

#[test]
fn the_median_of_an_even_count_is_the_lower_middle_run() {
    assert_eq!(
        measurement(&[4, 1, 3, 2]).median().unwrap(),
        Duration::from_millis(2)
    );
}

#[test]
fn a_measurement_without_runs_has_no_median() {
    assert!(matches!(measurement(&[]).median(), Err(Error::ZeroRuns)));
}

#[test]
fn the_report_renders_one_row_per_measurement() {
    let report = Report {
        fixture: "2 files".to_owned(),
        measurements: vec![measurement(&[3, 1, 2])],
    };
    assert_eq!(
        report.to_markdown().unwrap(),
        "Fixture: 2 files\n\n| Scenario | Backend | Runs | Median |\n| --- | --- | ---: | ---: |\n| import | git | 3 | 2.0 ms |\n"
    );
}
