//! Unit tests for the server client, against real and canned servers.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use openscion::PROTOCOL_VERSION;

use super::ServerClient;
use crate::Error;
use crate::test_support::{canned_server, real_server, unreachable_server};

#[test]
fn trailing_slashes_are_dropped_from_the_base_url() {
    assert_eq!(ServerClient::new("http://h:1///").base(), "http://h:1");
}

#[test]
fn info_reads_a_real_server() {
    let info = ServerClient::new(real_server()).info().unwrap();
    assert_eq!(info.protocol_version, PROTOCOL_VERSION);
    assert_eq!(info.server_version, env!("CARGO_PKG_VERSION"));
}

#[test]
fn an_unreachable_server_is_a_request_error() {
    let base = unreachable_server();
    let error = ServerClient::new(&base).info().unwrap_err();
    assert!(matches!(error, Error::Request { url, .. } if url == format!("{base}/v1/version")));
}

#[test]
fn an_error_status_is_a_request_error() {
    let error = ServerClient::new(canned_server(500, "{}"))
        .info()
        .unwrap_err();
    assert!(matches!(error, Error::Request { .. }));
}

#[test]
fn a_body_that_is_not_server_info_is_a_decode_error() {
    let error = ServerClient::new(canned_server(200, r#"{"hello":1}"#))
        .info()
        .unwrap_err();
    assert!(matches!(error, Error::Decode { .. }));
}

#[test]
fn a_server_on_another_major_protocol_is_rejected() {
    let base = canned_server(
        200,
        r#"{"server_version":"9.0.0","protocol_version":[99,0]}"#,
    );
    let error = ServerClient::new(base).info().unwrap_err();
    assert!(matches!(
        error,
        Error::IncompatibleProtocol { server: (99, 0) }
    ));
    assert_eq!(
        error.to_string(),
        format!(
            "server speaks protocol 99.0, which this client ({}.{}) cannot use",
            PROTOCOL_VERSION.0, PROTOCOL_VERSION.1
        )
    );
}
