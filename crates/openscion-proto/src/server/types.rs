//! The server description type.

use serde::{Deserialize, Serialize};

/// What a server reports about itself.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct ServerInfo {
    /// The server's package version, for display and diagnostics.
    pub server_version: String,
    /// The protocol version the server speaks, as `(major, minor)`.
    ///
    /// A client passes this to [`crate::is_compatible`] before making any
    /// other call.
    pub protocol_version: (u32, u32),
}

impl ServerInfo {
    /// Builds a description for a server at `server_version` speaking
    /// `protocol_version`.
    ///
    /// # Examples
    ///
    /// ```
    /// # use openscion_proto::{ServerInfo, PROTOCOL_VERSION};
    /// let info = ServerInfo::new("0.2.1", PROTOCOL_VERSION);
    /// assert_eq!(info.protocol_version, PROTOCOL_VERSION);
    /// ```
    #[must_use]
    pub fn new(server_version: impl Into<String>, protocol_version: (u32, u32)) -> Self {
        Self {
            server_version: server_version.into(),
            protocol_version,
        }
    }
}
