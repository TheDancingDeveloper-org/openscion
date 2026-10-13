//! The error body type.

use serde::{Deserialize, Serialize};

/// A machine-readable code and a human-readable message.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct ErrorBody {
    /// The error variant, such as `RefConflict` or `NotFound`.
    pub code: String,
    /// What went wrong, for people.
    pub message: String,
}

impl ErrorBody {
    /// Builds an error body.
    #[must_use]
    pub fn new(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            code: code.into(),
            message: message.into(),
        }
    }
}
