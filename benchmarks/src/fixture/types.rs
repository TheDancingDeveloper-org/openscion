//! Fixture types.

/// The size and layout of a synthetic repository.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FixtureShape {
    /// Number of top-level directories.
    pub directories: usize,
    /// Number of files in each directory.
    pub files_per_directory: usize,
    /// Size of every file, in bytes.
    pub file_bytes: usize,
}

/// One generated file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FixtureFile {
    /// Path relative to the repository root, `/`-separated.
    pub path: String,
    /// The file's contents.
    pub contents: Vec<u8>,
}

/// A generated repository: a list of files.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Fixture {
    pub(super) files: Vec<FixtureFile>,
}
