//! The ref payload types.

use serde::{Deserialize, Serialize};

use crate::ObjectId;

/// A ref's current value.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct RefValue {
    /// The ref's name, such as `main`.
    pub name: String,
    /// The changeset it points at.
    pub target: ObjectId,
    /// Starts at 1 and increases on every move.
    pub version: u64,
}

impl RefValue {
    /// Builds a ref value.
    #[must_use]
    pub fn new(name: impl Into<String>, target: ObjectId, version: u64) -> Self {
        Self {
            name: name.into(),
            target,
            version,
        }
    }
}

/// A compare-and-swap request for one ref.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct UpdateRef {
    /// The changeset the caller expects the ref to point at, or `None` if it
    /// expects the ref not to exist.
    pub expected: Option<ObjectId>,
    /// The changeset to point it at.
    pub new: ObjectId,
}

impl UpdateRef {
    /// Builds a swap from `expected` to `new`.
    #[must_use]
    pub fn new(expected: Option<ObjectId>, new: ObjectId) -> Self {
        Self { expected, new }
    }
}
