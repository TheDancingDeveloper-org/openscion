//! The protocol version, and the rule a client uses to decide whether it can bind
//! to a server that reports one.
//!
//! The version describes *this vocabulary*, not the crate: bump the major
//! component when a payload's wire form changes incompatibly or an operation is
//! removed or renamed, and the minor component when an operation or an optional
//! field is added. It is deliberately independent of the package version the
//! release workflow bumps, which tracks the shipped artifact.

/// The wire protocol version this crate defines.
pub const PROTOCOL_VERSION: (u32, u32) = (1, 0);

/// Returns whether a client holding [`PROTOCOL_VERSION`] can bind to a server
/// reporting `server`.
///
/// Compatibility is the ordinary semantic-version rule for a pre-release-free
/// contract: the majors must match, and the server must be at least as new as
/// the client, because a client cannot call an operation a server does not serve.
///
/// # Examples
///
/// ```
/// # use openscion_proto::{is_compatible, PROTOCOL_VERSION};
/// assert!(is_compatible(PROTOCOL_VERSION));
/// assert!(is_compatible((1, 4)));
/// assert!(!is_compatible((2, 0)));
/// ```
#[must_use]
pub fn is_compatible(server: (u32, u32)) -> bool {
    binds(PROTOCOL_VERSION, server)
}

/// The bind rule with the client version supplied explicitly.
///
/// [`is_compatible`] is this function applied to [`PROTOCOL_VERSION`]. It is
/// split out so the unit tests can exercise both directions of the comparison
/// without pinning them to whatever the shipped version happens to be.
fn binds(client: (u32, u32), server: (u32, u32)) -> bool {
    let (client_major, client_minor) = client;
    let (server_major, server_minor) = server;

    server_major == client_major && server_minor >= client_minor
}

#[cfg(test)]
mod test;
