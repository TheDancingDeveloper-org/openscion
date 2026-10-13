//! `scn`, the OpenScion command-line client.
//!
//! In the full design (`docs/specs/openscion-architecture.md`) `scn` lays out
//! a sparse working copy, watches it for edits, uploads blobs, and wraps the
//! server's history and landing APIs. Today it implements the commands the
//! server can already answer:
//!
//! | Command | What it does |
//! | --- | --- |
//! | `scn version` | prints the client and protocol versions |
//! | `scn server-info` | asks the server for its version and checks the protocol binds |
//!
//! Every command that talks to a server takes `--server <url>` or the
//! `OPENSCION_SERVER` environment variable, defaulting to
//! [`DEFAULT_SERVER`].
//!
//! # Example
//!
//! ```
//! use clap::Parser;
//! use openscion::{Cli, run};
//!
//! let cli = Cli::parse_from(["scn", "version"]);
//! let mut out = Vec::new();
//! run(&cli, &mut out)?;
//! assert!(String::from_utf8(out).unwrap().starts_with("scn "));
//! # Ok::<(), openscion::Error>(())
//! ```

mod cli;
mod client;
mod error;
#[cfg(test)]
mod test_support;

pub use cli::{Cli, Command, DEFAULT_SERVER, run};
pub use client::ServerClient;
pub use error::{Error, Result};
