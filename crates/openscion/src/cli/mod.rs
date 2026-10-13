//! Command-line parsing and dispatch.
//!
//! [`Cli`] is the parsed command line; [`run`] executes it and writes the
//! result to any [`Write`], so commands are tested without spawning a process.

use std::io::Write;

use clap::{Parser, Subcommand};
use openscion_core::PROTOCOL_VERSION;

use crate::{Error, Result, ServerClient};

/// The server `scn` talks to when neither `--server` nor `OPENSCION_SERVER`
/// is set: a local `openscion-server` on its default port.
pub const DEFAULT_SERVER: &str = "http://127.0.0.1:7878";

/// The parsed `scn` command line.
#[derive(Debug, Clone, PartialEq, Eq, Parser)]
#[command(name = "scn", version, about = "The OpenScion command-line client")]
#[non_exhaustive]
pub struct Cli {
    /// Base URL of the OpenScion server.
    #[arg(long, global = true, env = "OPENSCION_SERVER", default_value = DEFAULT_SERVER)]
    pub server: String,

    /// The command to run.
    #[command(subcommand)]
    pub command: Command,
}

/// The commands `scn` understands.
#[derive(Debug, Clone, PartialEq, Eq, Subcommand)]
#[non_exhaustive]
pub enum Command {
    /// Print the client and protocol versions.
    Version,
    /// Ask the server for its version and check that its protocol binds.
    ServerInfo,
}

/// Runs `cli`, writing its output to `out`.
///
/// # Errors
///
/// Returns the [`ServerClient`] error for commands that reach a server, and
/// [`Error::Output`] when writing to `out` fails.
pub fn run(cli: &Cli, out: &mut impl Write) -> Result<()> {
    match cli.command {
        Command::Version => {
            let (major, minor) = PROTOCOL_VERSION;
            writeln!(
                out,
                "scn {} (protocol {major}.{minor})",
                env!("CARGO_PKG_VERSION")
            )
            .map_err(Error::Output)
        }
        Command::ServerInfo => {
            let info = ServerClient::new(&cli.server).info()?;
            let (major, minor) = info.protocol_version;
            writeln!(
                out,
                "{}: openscion-server {} (protocol {major}.{minor})",
                cli.server, info.server_version
            )
            .map_err(Error::Output)
        }
    }
}

#[cfg(test)]
mod test;
