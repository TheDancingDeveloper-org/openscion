//! The JSON halves of batched object transfer.
//!
//! Object bytes travel in a binary frame stream, not JSON (the frame codec is
//! in the engine). The JSON side is small: a client names the objects it wants
//! with [`FetchObjects`], and the server acknowledges an upload with
//! [`StoredObjects`]. A client names exactly what it wants and never
//! negotiates what it already has (`docs/specs/openscion-api.md#fetch-protocol`).

mod types;

pub use types::{FetchObjects, StoredObjects};

#[cfg(test)]
mod test;
