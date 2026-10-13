//! Unit tests for server configuration parsing.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use clap::Parser;

use super::{DEFAULT_LISTEN, ServerConfig};

#[test]
fn the_default_listens_on_loopback() {
    assert_eq!(
        ServerConfig::default().listen,
        DEFAULT_LISTEN.parse().unwrap()
    );
}

#[test]
fn listen_is_read_from_the_command_line() {
    let config = ServerConfig::try_parse_from(["openscion-server", "--listen", "0.0.0.0:9000"])
        .unwrap();
    assert_eq!(config.listen, "0.0.0.0:9000".parse().unwrap());
}

#[test]
fn an_invalid_listen_address_is_rejected() {
    let parsed = ServerConfig::try_parse_from(["openscion-server", "--listen", "not-an-address"]);
    assert!(parsed.is_err());
}
