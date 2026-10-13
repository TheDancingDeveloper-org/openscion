//! Crate-wide error and result types for the benchmark harness.

use std::path::PathBuf;

/// Errors returned by the benchmark harness.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum Error {
    /// A filesystem operation on the benchmark work directory failed.
    #[error("cannot prepare {path}: {source}")]
    Io {
        /// The path being created, written, or removed.
        path: PathBuf,
        /// The underlying I/O error.
        #[source]
        source: std::io::Error,
    },
    /// An external command could not be started.
    #[error("cannot run {program}: {source}")]
    Spawn {
        /// The program that was invoked.
        program: String,
        /// The underlying I/O error.
        #[source]
        source: std::io::Error,
    },
    /// An external command ran and failed.
    #[error("{command} failed: {stderr}")]
    CommandFailed {
        /// The command line, for the report.
        command: String,
        /// What the command printed to standard error.
        stderr: String,
    },
    /// An `scn` command failed.
    #[error("scn: {0}")]
    OpenScion(#[from] openscion::Error),
    /// The in-process OpenScion server could not be started.
    #[error("cannot start openscion-server: {0}")]
    Server(String),
    /// The fixture has no directory for the sparse-clone scenario to fetch.
    #[error("the fixture has no directories to clone sparsely")]
    NoDirectory,
    /// The number of runs per scenario was zero.
    #[error("runs must be at least 1")]
    ZeroRuns,
}

/// The harness's standard result type.
pub type Result<T> = std::result::Result<T, Error>;
