//! Unit tests for object ids: the text form, its parser, and the JSON form.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use super::{ObjectId, ObjectKind, ParseIdError};

fn sample() -> ObjectId {
    let mut bytes = [0u8; 32];
    for (i, byte) in bytes.iter_mut().enumerate() {
        *byte = u8::try_from(i * 8).unwrap();
    }
    ObjectId::from_bytes(bytes)
}

#[test]
fn displays_as_lowercase_hex() {
    let text = sample().to_string();
    assert_eq!(text.len(), 64);
    assert!(text.starts_with("0008101820"));
    assert!(text.ends_with("f8"));
}

#[test]
fn parses_its_own_display() {
    let id = sample();
    assert_eq!(id.to_string().parse::<ObjectId>().unwrap(), id);
}

#[test]
fn rejects_the_wrong_length() {
    assert_eq!("abc".parse::<ObjectId>(), Err(ParseIdError::Length(3)));
}

#[test]
fn rejects_uppercase_hex() {
    let text = "A".repeat(64);
    assert_eq!(text.parse::<ObjectId>(), Err(ParseIdError::Digit('A')));
}

#[test]
fn rejects_non_hex_characters() {
    let text = "0".repeat(63) + "g";
    assert_eq!(text.parse::<ObjectId>(), Err(ParseIdError::Digit('g')));
}

#[test]
fn parse_errors_render_lowercase_messages() {
    assert_eq!(
        ParseIdError::Length(3).to_string(),
        "object id must be 64 characters, got 3"
    );
    assert_eq!(
        ParseIdError::Digit('g').to_string(),
        "object id holds 'g', not a lowercase hex digit"
    );
}

#[test]
fn serializes_to_a_hex_string() {
    let id = sample();
    assert_eq!(
        serde_json::to_value(id).unwrap(),
        serde_json::Value::String(id.to_string())
    );
    let decoded: ObjectId = serde_json::from_value(serde_json::to_value(id).unwrap()).unwrap();
    assert_eq!(decoded, id);
}

#[test]
fn rejects_a_malformed_json_id() {
    assert!(serde_json::from_value::<ObjectId>(serde_json::json!("xyz")).is_err());
    assert!(serde_json::from_value::<ObjectId>(serde_json::json!(7)).is_err());
}

#[test]
fn debug_shows_the_hex_form() {
    let id = sample();
    assert_eq!(format!("{id:?}"), format!("ObjectId({id})"));
}

#[test]
fn kinds_name_their_header_and_wire_form() {
    assert_eq!(ObjectKind::Blob.name(), "blob");
    assert_eq!(ObjectKind::Tree.name(), "tree");
    assert_eq!(ObjectKind::Changeset.name(), "changeset");
    assert_eq!(
        serde_json::to_value(ObjectKind::Changeset).unwrap(),
        serde_json::json!("changeset")
    );
}
