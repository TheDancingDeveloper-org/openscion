//! How engine errors become HTTP responses.

use axum::Json;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use openscion_core::{ErrorBody, ParseIdError};

/// A failed API call: an engine error or a malformed id in the URL.
///
/// | Error | Status |
/// | --- | --- |
/// | `NotFound` | `404` |
/// | `RefConflict` | `409` |
/// | anything the caller sent wrong | `400` |
#[derive(Debug)]
#[non_exhaustive]
pub enum ApiError {
    /// The engine rejected the call.
    Engine(openscion_core::Error),
    /// An id in the URL is not a valid object id.
    BadId(ParseIdError),
}

impl From<openscion_core::Error> for ApiError {
    fn from(error: openscion_core::Error) -> Self {
        Self::Engine(error)
    }
}

impl From<ParseIdError> for ApiError {
    fn from(error: ParseIdError) -> Self {
        Self::BadId(error)
    }
}

impl ApiError {
    /// The status code and body this error answers with.
    #[must_use]
    pub fn parts(&self) -> (StatusCode, ErrorBody) {
        match self {
            Self::Engine(error) => {
                let status = match error.code() {
                    "NotFound" => StatusCode::NOT_FOUND,
                    "RefConflict" => StatusCode::CONFLICT,
                    _ => StatusCode::BAD_REQUEST,
                };
                (status, ErrorBody::new(error.code(), error.to_string()))
            }
            Self::BadId(error) => (
                StatusCode::BAD_REQUEST,
                ErrorBody::new("InvalidArgument", error.to_string()),
            ),
        }
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, body) = self.parts();
        (status, Json(body)).into_response()
    }
}
