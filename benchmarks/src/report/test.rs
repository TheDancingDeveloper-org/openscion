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

fn on(backend: &'static str, scenario: Scenario, runs: &[u64]) -> Measurement {
    Measurement {
        backend,
        scenario,
        ..measurement(runs)
    }
}

#[test]
fn a_git_only_report_has_one_median_column() {
    let report = Report {
        fixture: "2 files".to_owned(),
        measurements: vec![measurement(&[3, 1, 2])],
    };
    assert_eq!(
        report.to_markdown().unwrap(),
        "Fixture: 2 files\n\n| Scenario | git |\n| --- | ---: |\n| import | 2.0 ms |\n"
    );
}

#[test]
fn other_backends_are_compared_with_git() {
    let report = Report {
        fixture: "f".to_owned(),
        measurements: vec![
            on("git", Scenario::Import, &[8]),
            on("openscion", Scenario::Import, &[2]),
            on("git", Scenario::FullClone, &[5]),
        ],
    };
    assert_eq!(
        report.to_markdown().unwrap(),
        "Fixture: f\n\n\
         | Scenario | git | openscion | openscion vs git |\n\
         | --- | ---: | ---: | ---: |\n\
         | import | 8.0 ms | 2.0 ms | 4.00× |\n\
         | full-clone | 5.0 ms | — | — |\n"
    );
}

#[test]
fn without_git_there_is_no_comparison_column() {
    let report = Report {
        fixture: "f".to_owned(),
        measurements: vec![on("openscion", Scenario::Import, &[0])],
    };
    assert!(
        report
            .to_markdown()
            .unwrap()
            .contains("| Scenario | openscion |\n")
    );
}

#[test]
fn a_zero_median_is_not_divided_by() {
    let report = Report {
        fixture: "f".to_owned(),
        measurements: vec![
            on("git", Scenario::Import, &[1]),
            on("openscion", Scenario::Import, &[0]),
        ],
    };
    assert!(report.to_markdown().unwrap().ends_with("| 0.0 ms | — |\n"));
}

#[test]
fn rendering_a_measurement_without_runs_fails() {
    let report = Report {
        fixture: "f".to_owned(),
        measurements: vec![measurement(&[])],
    };
    assert!(matches!(report.to_markdown(), Err(Error::ZeroRuns)));
}
