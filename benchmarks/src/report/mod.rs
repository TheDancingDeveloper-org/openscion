//! Collecting measurements and rendering them as a Markdown table.

use std::fmt::Write as _;
use std::time::Duration;

use crate::{Error, Result, Scenario};

/// All runs of one scenario on one backend.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Measurement {
    /// The backend that ran, such as `git`.
    pub backend: &'static str,
    /// The scenario that ran.
    pub scenario: Scenario,
    /// The duration of every run, in run order.
    pub runs: Vec<Duration>,
}

impl Measurement {
    /// The median run time; the lower middle value for an even count.
    ///
    /// # Errors
    ///
    /// Returns [`Error::ZeroRuns`] when there are no runs.
    pub fn median(&self) -> Result<Duration> {
        let mut sorted = self.runs.clone();
        sorted.sort_unstable();
        sorted
            .get(sorted.len().saturating_sub(1) / 2)
            .copied()
            .ok_or(Error::ZeroRuns)
    }
}

/// A full benchmark run: what was measured and the fixture it ran on.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Report {
    /// One-line description of the fixture, printed above the table.
    pub fixture: String,
    /// Every measurement, in the order it was taken.
    pub measurements: Vec<Measurement>,
}

impl Report {
    /// Renders the report as a Markdown table: one row per scenario, one
    /// median column per backend, and — when `git` is one of the backends —
    /// how many times faster each other backend is than git.
    ///
    /// # Errors
    ///
    /// Returns [`Error::ZeroRuns`] when a measurement has no runs.
    pub fn to_markdown(&self) -> Result<String> {
        let mut backends: Vec<&str> = Vec::new();
        let mut scenarios: Vec<Scenario> = Vec::new();
        for measurement in &self.measurements {
            if !backends.contains(&measurement.backend) {
                backends.push(measurement.backend);
            }
            if !scenarios.contains(&measurement.scenario) {
                scenarios.push(measurement.scenario);
            }
        }
        let others: Vec<&str> = if backends.contains(&"git") {
            backends.iter().copied().filter(|b| *b != "git").collect()
        } else {
            Vec::new()
        };

        // Writing to a `String` cannot fail, so `write!` results are ignored.
        let mut out = format!("Fixture: {}\n\n| Scenario |", self.fixture);
        let mut rule = String::from("| --- |");
        for backend in &backends {
            let _ = write!(out, " {backend} |");
            rule.push_str(" ---: |");
        }
        for backend in &others {
            let _ = write!(out, " {backend} vs git |");
            rule.push_str(" ---: |");
        }
        let _ = writeln!(out, "\n{rule}");

        for scenario in scenarios {
            let _ = write!(out, "| {scenario} |");
            let mut medians = Vec::with_capacity(backends.len());
            for backend in &backends {
                let median = self.median(scenario, backend)?;
                match median {
                    Some(median) => {
                        let _ = write!(out, " {:.1} ms |", millis(median));
                    }
                    None => out.push_str(" — |"),
                }
                medians.push((*backend, median));
            }
            let git = medians
                .iter()
                .find(|(backend, _)| *backend == "git")
                .and_then(|(_, median)| *median);
            for backend in &others {
                let other = medians
                    .iter()
                    .find(|(name, _)| name == backend)
                    .and_then(|(_, median)| *median);
                match (git, other) {
                    (Some(git), Some(other)) if !other.is_zero() => {
                        let _ = write!(out, " {:.2}× |", millis(git) / millis(other));
                    }
                    _ => out.push_str(" — |"),
                }
            }
            out.push('\n');
        }
        Ok(out)
    }

    /// The median of `scenario` on `backend`, or `None` if it was not run.
    fn median(&self, scenario: Scenario, backend: &str) -> Result<Option<Duration>> {
        self.measurements
            .iter()
            .find(|m| m.scenario == scenario && m.backend == backend)
            .map(Measurement::median)
            .transpose()
    }
}

fn millis(duration: Duration) -> f64 {
    duration.as_secs_f64() * 1000.0
}

#[cfg(test)]
mod test;
