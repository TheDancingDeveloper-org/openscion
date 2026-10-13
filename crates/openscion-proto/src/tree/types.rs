//! The tree entry types.

use serde::{Deserialize, Serialize};

use crate::ObjectId;

/// What a tree entry points at.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EntryKind {
    /// A regular file; the entry's id is a blob.
    File,
    /// An executable file; the entry's id is a blob.
    Executable,
    /// A symbolic link; the entry's id is a blob holding the link target.
    Symlink,
    /// A directory; the entry's id is a tree.
    Tree,
}

/// One entry in a tree.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct TreeEntry {
    /// A single path component, unique within its tree.
    pub name: String,
    /// What the entry points at.
    pub kind: EntryKind,
    /// The blob id for files and symlinks, the tree id for directories.
    pub id: ObjectId,
}

impl TreeEntry {
    /// Builds an entry.
    #[must_use]
    pub fn new(name: impl Into<String>, kind: EntryKind, id: ObjectId) -> Self {
        Self {
            name: name.into(),
            kind,
            id,
        }
    }
}
