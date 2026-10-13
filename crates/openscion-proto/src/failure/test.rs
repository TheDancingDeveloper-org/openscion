//! Unit tests pinning the error body wire form.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use super::ErrorBody;

#[test]
fn error_body_serializes_to_its_wire_form() {
    let body = ErrorBody::new("NotFound", "no such object");
    let encoded = serde_json::to_value(&body).unwrap();
    assert_eq!(
        encoded,
        serde_json::json!({ "code": "NotFound", "message": "no such object" })
    );
    assert_eq!(serde_json::from_value::<ErrorBody>(encoded).unwrap(), body);
}
