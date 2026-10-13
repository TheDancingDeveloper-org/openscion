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
    /// Renders the report as a Markdown table of median times.
    ///
    /// # Errors
    ///
    /// Returns [`Error::ZeroRuns`] when a measurement has no runs.
    pub fn to_markdown(&self) -> Result<String> {
        let mut out = format!(
            "Fixture: {}\n\n| Scenario | Backend | Runs | Median |\n| --- | --- | ---: | ---: |\n",
            self.fixture
        );
        for measurement in &self.measurements {
            let median = measurement.median()?;
            // Writing to a `String` cannot fail.
            let _ = writeln!(
                out,
                "| {} | {} | {} | {:.1} ms |",
                measurement.scenario,
                measurement.backend,
                measurement.runs.len(),
                median.as_secs_f64() * 1000.0
            );
        }
        Ok(out)
    }
}

#[cfg(test)]
mod test;
