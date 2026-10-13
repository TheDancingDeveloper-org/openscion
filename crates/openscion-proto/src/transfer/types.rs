//! The object transfer payload types.

use serde::{Deserialize, Serialize};

use crate::ObjectId;

/// A request for a batch of trees and blobs by id.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct FetchObjects {
    /// The objects wanted, in the order they should be returned.
    pub ids: Vec<ObjectId>,
}

impl FetchObjects {
    /// Builds a fetch request.
    #[must_use]
    pub fn new(ids: Vec<ObjectId>) -> Self {
        Self { ids }
    }
}

/// The ids the server computed for an uploaded batch, in upload order.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct StoredObjects {
    /// One id per uploaded object.
    pub ids: Vec<ObjectId>,
}

impl StoredObjects {
    /// Builds an upload acknowledgement.
    #[must_use]
    pub fn new(ids: Vec<ObjectId>) -> Self {
        Self { ids }
    }
}
