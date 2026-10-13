//! Object hashing: how an object's bytes become its [`ObjectId`].
//!
//! ```text
//! id = BLAKE3( kind ‖ 0x20 ‖ decimal(len(payload)) ‖ 0x00 ‖ payload )
//! ```
//!
//! The header binds the kind and length, so a blob and a tree that share bytes
//! still get different ids (`docs/specs/openscion-data-model.md#object-identifiers`).

use openscion_proto::{ObjectId, ObjectKind};

/// Computes the id of an object of `kind` whose canonical encoding is
/// `payload`.
///
/// # Examples
///
/// ```
/// use openscion_core::{ObjectKind, object_id};
///
/// let blob = object_id(ObjectKind::Blob, b"hello\n");
/// assert_ne!(blob, object_id(ObjectKind::Tree, b"hello\n"));
/// assert_eq!(blob, object_id(ObjectKind::Blob, b"hello\n"));
/// ```
#[must_use]
pub fn object_id(kind: ObjectKind, payload: &[u8]) -> ObjectId {
    let mut hasher = blake3::Hasher::new();
    hasher.update(kind.name().as_bytes());
    hasher.update(b" ");
    hasher.update(payload.len().to_string().as_bytes());
    hasher.update(&[0]);
    hasher.update(payload);
    ObjectId::from_bytes(*hasher.finalize().as_bytes())
}

#[cfg(test)]
mod test;
