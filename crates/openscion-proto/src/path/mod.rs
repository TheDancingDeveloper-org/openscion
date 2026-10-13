//! Repository paths.
//!
//! A [`RepoPath`] names a file relative to the repository root
//! (`docs/specs/openscion-data-model.md#paths`): UTF-8, `/`-separated, with no
//! empty, `.`, or `..` component, no NUL, components of at most 255 bytes, and
//! at most 4096 bytes in all. Paths order by their UTF-8 bytes. A `RepoPath`
//! can only be built through validation, so holding one proves it is valid.

mod types;

pub use types::{PathError, RepoPath};

#[cfg(test)]
mod test;
