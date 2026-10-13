//! Unit tests pinning the transfer payloads' wire forms.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use super::{FetchObjects, StoredObjects};
use crate::ObjectId;

#[test]
fn fetch_and_store_carry_a_list_of_ids() {
    let id = ObjectId::from_bytes([5; 32]);
    let expected = serde_json::json!({ "ids": [id.to_string()] });
    assert_eq!(
        serde_json::to_value(FetchObjects::new(vec![id])).unwrap(),
        expected
    );
    assert_eq!(
        serde_json::to_value(StoredObjects::new(vec![id])).unwrap(),
        expected
    );
    assert_eq!(
        serde_json::from_value::<FetchObjects>(expected.clone()).unwrap().ids,
        [id]
    );
    assert_eq!(
        serde_json::from_value::<StoredObjects>(expected).unwrap().ids,
        [id]
    );
}
