//! End-to-end benchmarks comparing OpenScion with git.
//!
//! The harness generates a deterministic synthetic repository (a
//! [`Fixture`]), runs each [`Scenario`] against every backend a fixed number
//! of times, and reports the median wall-clock time per scenario and backend.
//!
//! These are coarse, whole-operation timings — seconds, not nanoseconds — of
//! the things a developer or CI job actually waits for: importing a tree,
//! committing a change, and getting a full copy. They are meant to show trends
//! between versions and against git, not to resolve small differences.
//!
//! # Backends
//!
//! - [`git`] drives the `git` command-line tool, which must be on `PATH`.
//! - The OpenScion backend is added when the engine can create changesets
//!   (phase 1 in `docs/specs/openscion-architecture.md`); until then the
//!   harness records the git baselines OpenScion will be measured against.
//!
//! # Example
//!
//! ```
//! use openscion_benchmarks::{Fixture, FixtureShape};
//!
//! let shape = FixtureShape { directories: 2, files_per_directory: 3, file_bytes: 64 };
//! let fixture = Fixture::generate(shape, 7);
//! assert_eq!(fixture.files().len(), 6);
//! assert_eq!(fixture, Fixture::generate(shape, 7));
//! ```

mod error;
mod fixture;
pub mod git;
mod report;
mod scenario;

pub use error::{Error, Result};
pub use fixture::{Fixture, FixtureFile, FixtureShape};
pub use report::{Measurement, Report};
pub use scenario::Scenario;
