//! Crate-wide error and result types for the client.

use openscion_core::PROTOCOL_VERSION;

/// Errors returned by `scn`.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum Error {
    /// The server could not be reached, or answered with an error status.
    #[error("request to {url} failed: {message}")]
    Request {
        /// The URL that was requested.
        url: String,
        /// What went wrong, as reported by the HTTP client.
        message: String,
    },
    /// The server answered, but not with the payload the protocol defines.
    #[error("unexpected response from {url}: {message}")]
    Decode {
        /// The URL that was requested.
        url: String,
        /// Why the body could not be decoded.
        message: String,
    },
    /// The server speaks a protocol version this client cannot bind to.
    #[error(
        "server speaks protocol {}.{}, which this client ({}.{}) cannot use",
        server.0, server.1, PROTOCOL_VERSION.0, PROTOCOL_VERSION.1
    )]
    IncompatibleProtocol {
        /// The server's protocol version.
        server: (u32, u32),
    },
    /// Writing command output failed.
    #[error("cannot write output: {0}")]
    Output(#[source] std::io::Error),
}

/// The client's standard result type.
pub type Result<T> = std::result::Result<T, Error>;
