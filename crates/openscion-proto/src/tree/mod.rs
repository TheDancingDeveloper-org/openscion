//! Tree entries: what one directory holds.
//!
//! A tree lists one directory's entries, each a name, an [`EntryKind`], and the
//! id of the blob or tree it points at
//! (`docs/specs/openscion-data-model.md#trees`). Trees are derived by the
//! server and fetched by clients; their canonical byte encoding lives in the
//! engine.

mod types;

pub use types::{EntryKind, TreeEntry};

#[cfg(test)]
mod test;
