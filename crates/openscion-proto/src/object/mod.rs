//! Object identifiers: the names of every blob, tree, and changeset.
//!
//! An [`ObjectId`] is the 32-byte BLAKE3 hash of an object's canonical
//! encoding, written as 64 lowercase hexadecimal characters
//! (`docs/specs/openscion-data-model.md#object-identifiers`). This module only
//! parses and prints ids; computing one is the engine's job, so this crate
//! carries no hash implementation.

mod types;

pub use types::{ObjectId, ObjectKind, ParseIdError};

#[cfg(test)]
mod test;
