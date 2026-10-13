//! Request and response types for the placeholder `Greet` operation.

use serde::{Deserialize, Serialize};

/// The request for the placeholder `Greet` operation.
///
/// The engine trims surrounding whitespace from [`GreetRequest::name`] and
/// rejects a name that is empty once trimmed.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct GreetRequest {
    /// The name to greet.
    pub name: String,
}

impl GreetRequest {
    /// Builds a request greeting `name`.
    ///
    /// # Examples
    ///
    /// ```
    /// # use openscion_proto::GreetRequest;
    /// assert_eq!(GreetRequest::new("Ferris").name, "Ferris");
    /// ```
    #[must_use]
    pub fn new(name: impl Into<String>) -> Self {
        Self { name: name.into() }
    }
}

/// The response to the placeholder `Greet` operation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct GreetResponse {
    /// The rendered greeting.
    pub greeting: String,
}

impl GreetResponse {
    /// Builds a reply carrying `greeting`.
    ///
    /// # Examples
    ///
    /// ```
    /// # use openscion_proto::GreetResponse;
    /// assert_eq!(GreetResponse::new("Hello, Ferris!").greeting, "Hello, Ferris!");
    /// ```
    #[must_use]
    pub fn new(greeting: impl Into<String>) -> Self {
        Self {
            greeting: greeting.into(),
        }
    }
}
