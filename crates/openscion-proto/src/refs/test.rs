//! Unit tests pinning the ref payloads' wire forms.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use super::{RefValue, UpdateRef};
use crate::ObjectId;

#[test]
fn ref_value_serializes_to_its_wire_form() {
    let id = ObjectId::from_bytes([3; 32]);
    let value = RefValue::new("main", id, 2);
    let encoded = serde_json::to_value(&value).unwrap();
    assert_eq!(
        encoded,
        serde_json::json!({ "name": "main", "target": id.to_string(), "version": 2 })
    );
    assert_eq!(serde_json::from_value::<RefValue>(encoded).unwrap(), value);
}

#[test]
fn update_ref_encodes_an_absent_expectation_as_null() {
    let id = ObjectId::from_bytes([4; 32]);
    let update = UpdateRef::new(None, id);
    let encoded = serde_json::to_value(update).unwrap();
    assert_eq!(
        encoded,
        serde_json::json!({ "expected": null, "new": id.to_string() })
    );
    assert_eq!(serde_json::from_value::<UpdateRef>(encoded).unwrap(), update);
}
