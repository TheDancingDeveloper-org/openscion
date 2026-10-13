//! Refs: named pointers into the log that move only by compare-and-swap.
//!
//! A ref maps a name to a changeset id and carries a version that increases
//! on every move (`docs/specs/openscion-data-model.md#refs`). A caller moves
//! one with an [`UpdateRef`] that states what it expects the ref to hold; the
//! server answers with the new [`RefValue`] or a conflict.

mod types;

pub use types::{RefValue, UpdateRef};

#[cfg(test)]
mod test;
