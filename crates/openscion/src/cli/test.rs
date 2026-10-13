//! Unit tests for command parsing and dispatch.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::io::{self, Write};

use clap::Parser;
use openscion_core::PROTOCOL_VERSION;

use super::{Cli, Command, DEFAULT_SERVER, run};
use crate::Error;
use crate::test_support::{real_server, unreachable_server};

fn output(cli: &Cli) -> String {
    let mut out = Vec::new();
    run(cli, &mut out).unwrap();
    String::from_utf8(out).unwrap()
}

#[test]
fn the_server_defaults_to_a_local_openscion_server() {
    let cli = Cli::try_parse_from(["scn", "version"]).unwrap();
    assert_eq!(cli.server, DEFAULT_SERVER);
    assert_eq!(cli.command, Command::Version);
}

#[test]
fn server_is_accepted_after_the_subcommand() {
    let cli = Cli::try_parse_from(["scn", "server-info", "--server", "http://h:1"]).unwrap();
    assert_eq!(cli.server, "http://h:1");
    assert_eq!(cli.command, Command::ServerInfo);
}

#[test]
fn an_unknown_command_is_rejected() {
    assert!(Cli::try_parse_from(["scn", "frobnicate"]).is_err());
}

#[test]
fn version_prints_the_client_and_protocol_versions() {
    let cli = Cli::try_parse_from(["scn", "version"]).unwrap();
    let (major, minor) = PROTOCOL_VERSION;
    assert_eq!(
        output(&cli),
        format!(
            "scn {} (protocol {major}.{minor})\n",
            env!("CARGO_PKG_VERSION")
        )
    );
}

#[test]
fn server_info_prints_what_the_server_reports() {
    let base = real_server();
    let cli = Cli::try_parse_from(["scn", "--server", &base, "server-info"]).unwrap();
    let (major, minor) = PROTOCOL_VERSION;
    assert_eq!(
        output(&cli),
        format!(
            "{base}: openscion-server {} (protocol {major}.{minor})\n",
            env!("CARGO_PKG_VERSION")
        )
    );
}

#[test]
fn server_info_fails_when_the_server_is_unreachable() {
    let cli =
        Cli::try_parse_from(["scn", "--server", &unreachable_server(), "server-info"]).unwrap();
    assert!(matches!(
        run(&cli, &mut Vec::new()),
        Err(Error::Request { .. })
    ));
}

struct BrokenPipe;

impl Write for BrokenPipe {
    fn write(&mut self, _: &[u8]) -> io::Result<usize> {
        Err(io::Error::from(io::ErrorKind::BrokenPipe))
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

#[test]
fn a_failed_write_is_an_output_error() {
    let cli = Cli::try_parse_from(["scn", "version"]).unwrap();
    let error = run(&cli, &mut BrokenPipe).unwrap_err();
    assert!(matches!(error, Error::Output(_)));
    assert!(error.to_string().starts_with("cannot write output: "));

    let cli = Cli::try_parse_from(["scn", "--server", &real_server(), "server-info"]).unwrap();
    assert!(matches!(run(&cli, &mut BrokenPipe), Err(Error::Output(_))));
}
