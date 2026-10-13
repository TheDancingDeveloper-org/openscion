//! Deterministic synthetic repositories.
//!
//! A fixture is a list of files with generated contents. The same shape and
//! seed always produce byte-identical files, so two runs — or two machines —
//! benchmark exactly the same input.

mod types;

use std::path::Path;

pub use types::{Fixture, FixtureFile, FixtureShape};

use crate::{Error, Result};

impl Fixture {
    /// Generates the fixture for `shape` from `seed`.
    #[must_use]
    pub fn generate(shape: FixtureShape, seed: u64) -> Self {
        let mut state = seed ^ 0x9E37_79B9_7F4A_7C15;
        let mut files = Vec::with_capacity(shape.directories * shape.files_per_directory);
        for directory in 0..shape.directories {
            for file in 0..shape.files_per_directory {
                let contents = (0..shape.file_bytes)
                    .map(|_| {
                        state = next(state);
                        // Printable ASCII keeps the files diffable text, which
                        // is what source repositories mostly hold.
                        b' ' + u8::try_from((state >> 33) % 95).unwrap_or(0)
                    })
                    .collect();
                files.push(FixtureFile {
                    path: format!("dir{directory:04}/file{file:04}.txt"),
                    contents,
                });
            }
        }
        Self { files }
    }

    /// The fixture's files, in path order.
    #[must_use]
    pub fn files(&self) -> &[FixtureFile] {
        &self.files
    }

    /// Writes every file under `root`, creating directories as needed.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Io`] when a directory or file cannot be written.
    pub fn write_to(&self, root: &Path) -> Result<()> {
        for file in &self.files {
            let path = root.join(&file.path);
            if let Some(parent) = path.parent() {
                std::fs::create_dir_all(parent).map_err(|source| Error::Io {
                    path: parent.to_path_buf(),
                    source,
                })?;
            }
            std::fs::write(&path, &file.contents).map_err(|source| Error::Io { path, source })?;
        }
        Ok(())
    }
}

/// One step of a 64-bit linear congruential generator (Knuth's MMIX
/// constants). Statistical quality is irrelevant here; determinism is the
/// point, and it needs no dependency.
fn next(state: u64) -> u64 {
    state
        .wrapping_mul(6_364_136_223_846_793_005)
        .wrapping_add(1_442_695_040_888_963_407)
}

#[cfg(test)]
mod test;
