//! The OpenScion protocol: every payload type that crosses a process boundary.
//!
//! OpenScion is a server-centric source control system: the server owns history
//! and computes commits, rebases, and queries, while clients fetch exactly the
//! objects they need. Every component — the engine, the node daemon, the
//! regional cache, the server, and the `scn` client — speaks the same API. This
//! crate is that API's vocabulary, so a tool that only makes calls depends on
//! it alone. See `docs/specs/openscion-api.md` for the operations it carries.
//!
//! # What is here
//!
//! - [`greeting`] — a placeholder payload family kept from the project
//!   template. Phase 1 replaces it with object ids, changesets, and refs.
//! - [`server`] — [`ServerInfo`], what a server reports about itself.
//! - [`version`] — [`PROTOCOL_VERSION`] and the [`is_compatible`] bind rule.
//!
//! # What is deliberately not here
//!
//! **No behavior.** Hashing, validation, and every operation live in
//! `crates/openscion-core`, which depends on this crate and re-exports it. A payload
//! type describes what a request carries, not what the server does with it.
//!
//! **No transport.** This crate holds no connection, client, or codec, and
//! depends on no async runtime, HTTP client, or native library. CI asserts it.
//! Whatever transport carries these payloads — a local socket to the daemon or
//! a network hop between tiers — it carries the same types.
//!
//! # This crate sits underneath the engine, not beside it
//!
//! `openscion-core` **depends on this crate and re-exports all of it**, so
//! `openscion_core::GreetRequest` and `openscion_proto::greeting::GreetRequest` are
//! the *same type*, not structural twins. A parallel set of payload types for
//! clients would mean a conversion at every call site that nothing checks.
//!
//! # Example
//!
//! ```
//! use openscion_proto::{GreetRequest, GreetResponse, PROTOCOL_VERSION, is_compatible};
//!
//! let body = serde_json::to_value(GreetRequest::new("Ferris"))?;
//! assert_eq!(body, serde_json::json!({ "name": "Ferris" }));
//! assert!(is_compatible(PROTOCOL_VERSION));
//!
//! let reply: GreetResponse = serde_json::from_value(
//!     serde_json::json!({ "greeting": "Hello, Ferris!" }),
//! )?;
//! assert_eq!(reply.greeting, "Hello, Ferris!");
//! # Ok::<(), serde_json::Error>(())
//! ```

pub mod greeting;
pub mod server;
pub mod version;

pub use greeting::{GreetRequest, GreetResponse};
pub use server::ServerInfo;
pub use version::{PROTOCOL_VERSION, is_compatible};
