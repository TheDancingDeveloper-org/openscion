//! The operations the harness times.
//!
//! Each scenario names one thing a developer or CI job waits for. Setup that
//! the scenario does not measure — writing the fixture, creating the history
//! it starts from — happens before the clock starts.

use std::fmt;

/// One timed operation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum Scenario {
    /// Record a whole working tree as the first commit or changeset.
    Import,
    /// Change one file in an imported tree and record that change.
    CommitOneFile,
    /// Produce a complete local copy of an imported repository.
    FullClone,
    /// Produce a local copy of one top-level directory of an imported
    /// repository — the agent case, where a job needs a slice of the tree.
    SparseClone,
}

impl Scenario {
    /// Every scenario, in report order.
    pub const ALL: [Self; 4] = [
        Self::Import,
        Self::CommitOneFile,
        Self::FullClone,
        Self::SparseClone,
    ];

    /// The scenario's name in reports and on the command line.
    #[must_use]
    pub fn name(self) -> &'static str {
        match self {
            Self::Import => "import",
            Self::CommitOneFile => "commit-one-file",
            Self::FullClone => "full-clone",
            Self::SparseClone => "sparse-clone",
        }
    }
}

impl fmt::Display for Scenario {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.name())
    }
}

#[cfg(test)]
mod test;
