//! Canonical byte encodings: what an id is computed over, and how objects
//! travel in bulk.
//!
//! - Trees encode as their entries sorted by name, each
//!   `kind-byte ‖ name ‖ 0x00 ‖ id`, with kind bytes `f`, `x`, `l`, `d`
//!   (`docs/specs/openscion-data-model.md#trees`).
//! - Changesets encode in format version 1: big-endian integers and
//!   length-prefixed strings (`docs/specs/openscion-data-model.md#canonical-encoding`).
//! - Batches of objects travel as a stream of frames, `id ‖ u64 length ‖ bytes`,
//!   which the server and the client both read and write.
//!
//! Decoders validate everything they read, so a malformed or non-canonical
//! encoding is rejected rather than half-read.

mod changeset;
mod frame;
mod tree;

pub use changeset::encode_changeset;
pub use frame::{FrameWriter, decode_frames};
pub use tree::{decode_tree, encode_tree};

#[cfg(test)]
mod test;
