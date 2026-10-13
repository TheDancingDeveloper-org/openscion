//! Unit tests pinning the tree entry wire form.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use super::{EntryKind, TreeEntry};
use crate::ObjectId;

#[test]
fn entry_serializes_to_its_wire_form() {
    let id = ObjectId::from_bytes([1; 32]);
    let entry = TreeEntry::new("lib.rs", EntryKind::Executable, id);
    assert_eq!(
        serde_json::to_value(&entry).unwrap(),
        serde_json::json!({ "name": "lib.rs", "kind": "executable", "id": id.to_string() })
    );
    let decoded: TreeEntry = serde_json::from_value(serde_json::to_value(&entry).unwrap()).unwrap();
    assert_eq!(decoded, entry);
}

#[test]
fn kinds_use_snake_case_names() {
    let names: Vec<_> = [
        EntryKind::File,
        EntryKind::Executable,
        EntryKind::Symlink,
        EntryKind::Tree,
    ]
    .iter()
    .map(|kind| serde_json::to_value(kind).unwrap())
    .collect();
    assert_eq!(
        names,
        [
            serde_json::json!("file"),
            serde_json::json!("executable"),
            serde_json::json!("symlink"),
            serde_json::json!("tree")
        ]
    );
}
