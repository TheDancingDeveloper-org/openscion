//! The OpenScion server: the authoritative tier of the system.
//!
//! In the architecture (`docs/specs/openscion-architecture.md`) the server owns
//! the changeset log, the refs, and the derived-data indexes, and is the only
//! component that creates changesets or moves refs. Today it serves the
//! discovery endpoints every client calls first; the API operations arrive
//! with the phases that implement them in the engine.
//!
//! # Endpoints
//!
//! | Method | Path | Response |
//! | --- | --- | --- |
//! | `GET` | `/healthz` | `200 ok` while the process is serving |
//! | `GET` | `/v1/version` | [`ServerInfo`] as JSON |
//!
//! # Entry points
//!
//! - [`ServerConfig`] — the command-line and environment configuration.
//! - [`Server::bind`] then [`Server::serve`] — listen and serve until a
//!   shutdown future resolves.
//! - [`router`] — the HTTP routes, for embedding or testing.
//!
//! # Example
//!
//! ```no_run
//! use openscion_server::{Server, ServerConfig};
//!
//! # async fn run() -> openscion_server::Result<()> {
//! let config = ServerConfig::default();
//! let server = Server::bind(&config).await?;
//! println!("listening on {}", server.local_addr());
//! server.serve(std::future::pending()).await
//! # }
//! ```
//!
//! # What is deliberately not here
//!
//! No source control logic. Hashing, validation, the log, and refs live in the
//! `openscion` engine; this crate translates HTTP to engine calls and back.

mod config;
mod error;
mod http;

pub use config::{DEFAULT_LISTEN, ServerConfig};
pub use error::{Error, Result};
pub use http::{Server, router, shutdown_signal};
pub use openscion_core::ServerInfo;
