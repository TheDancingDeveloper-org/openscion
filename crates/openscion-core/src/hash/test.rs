//! Unit tests for object hashing.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use super::object_id;
use openscion_proto::ObjectKind;

#[test]
fn hashes_the_header_and_payload() {
    let expected = blake3::hash(b"blob 5\0hello");
    assert_eq!(
        object_id(ObjectKind::Blob, b"hello").as_bytes(),
        expected.as_bytes()
    );
}

#[test]
fn the_empty_tree_has_a_fixed_id() {
    let expected = blake3::hash(b"tree 0\0");
    assert_eq!(
        object_id(ObjectKind::Tree, b"").as_bytes(),
        expected.as_bytes()
    );
}

#[test]
fn kind_changes_the_id() {
    let payload = b"same bytes";
    assert_ne!(
        object_id(ObjectKind::Blob, payload),
        object_id(ObjectKind::Changeset, payload)
    );
}
