//! A blocking HTTP client for the OpenScion server API.
//!
//! Each method maps to one endpoint and returns the decoded protocol payload.
//! The client checks the protocol version on [`ServerClient::info`], so a
//! caller that starts there never talks to a server it cannot understand.

use openscion::{ServerInfo, is_compatible};

use crate::{Error, Result};

/// A client for one OpenScion server.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServerClient {
    base: String,
}

impl ServerClient {
    /// Creates a client for the server at `base`, such as
    /// `http://127.0.0.1:7878`. A trailing `/` is ignored.
    ///
    /// # Examples
    ///
    /// ```
    /// # use openscion_cli::ServerClient;
    /// let client = ServerClient::new("http://127.0.0.1:7878/");
    /// assert_eq!(client.base(), "http://127.0.0.1:7878");
    /// ```
    #[must_use]
    pub fn new(base: impl Into<String>) -> Self {
        let mut base = base.into();
        while base.ends_with('/') {
            base.pop();
        }
        Self { base }
    }

    /// The server's base URL, without a trailing `/`.
    #[must_use]
    pub fn base(&self) -> &str {
        &self.base
    }

    /// Fetches `GET /v1/version` and checks that the protocol binds.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Request`] when the server cannot be reached or answers
    /// with an error status, [`Error::Decode`] when the body is not a
    /// [`ServerInfo`], and [`Error::IncompatibleProtocol`] when the server's
    /// protocol version does not bind to this client's.
    pub fn info(&self) -> Result<ServerInfo> {
        let url = format!("{}/v1/version", self.base);
        let mut response = ureq::get(&url).call().map_err(|error| Error::Request {
            url: url.clone(),
            message: error.to_string(),
        })?;
        let info: ServerInfo = response
            .body_mut()
            .read_json()
            .map_err(|error| Error::Decode {
                url: url.clone(),
                message: error.to_string(),
            })?;
        if !is_compatible(info.protocol_version) {
            return Err(Error::IncompatibleProtocol {
                server: info.protocol_version,
            });
        }
        Ok(info)
    }
}

#[cfg(test)]
mod test;
