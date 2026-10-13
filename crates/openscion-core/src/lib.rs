//! The OpenScion engine: a server-centric, content-addressed source control
//! system for monorepos whose clients are mostly always-online agents and jobs.
//!
//! The design is specified in `docs/specs/`: the architecture, the data model
//! (objects, changesets, the log, refs, derived data), and the API. This crate
//! will hold the engine those specs describe — hashing, validation, the log and
//! ref stores behind their ports, derived data, and server-side rebase — as a
//! plain library usable in-process with no transport.
//!
//! # Layout
//!
//! This is the engine half of a two-crate workspace:
//!
//! - [`openscion_proto`] — the protocol. Payload types and the protocol
//!   version, with no transport and no behavior. A client that only makes calls
//!   depends on that crate alone.
//! - `openscion` — this crate. The behavior and the crate-wide error type.
//!
//! Within this crate:
//!
//! - `src/error/` holds the crate-wide [`Error`] enum and the [`Result`] alias
//!   returned by every fallible public function.
//! - Each feature area lives in its own module directory with a `mod.rs`
//!   module root, an optional `types.rs`, and a `test.rs` holding its unit
//!   tests.
//! - Every public item is re-exported from here — including all of
//!   [`openscion_proto`] — so downstream users have a single predictable
//!   surface and `openscion_core::GreetRequest` is the *same type* as
//!   `openscion_proto::GreetRequest`.
//!
//! # Example
//!
//! The only behavior today is the placeholder kept from the project template;
//! phase 1 of the architecture spec replaces it with the core data model.
//!
//! ```
//! use openscion_core::{greet, Error, GreetRequest};
//!
//! assert_eq!(greet("Ferris")?, "Hello, Ferris!");
//! assert_eq!(greet("   ").unwrap_err(), Error::EmptyName);
//! assert_eq!(GreetRequest::new("Ferris").name, "Ferris");
//! # Ok::<(), openscion_core::Error>(())
//! ```

mod error;
mod greeting;

pub use error::{Error, Result};
pub use greeting::greet;

// The protocol, re-exported by module rather than by item so every path
// through this crate resolves to the same definitions the protocol crate
// publishes. A client may depend on `openscion-proto` directly and get exactly
// these types; nothing here redefines them.
pub use openscion_proto;
pub use openscion_proto::{
    GreetRequest, GreetResponse, PROTOCOL_VERSION, ServerInfo, is_compatible, server, version,
};
