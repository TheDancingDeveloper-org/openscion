//! The body every failed request carries.
//!
//! A server answers a failed call with an HTTP status and an [`ErrorBody`]
//! whose `code` names the error variant from
//! `docs/specs/openscion-api.md#errors`, so a client can branch on the code
//! without parsing the message.

mod types;

pub use types::ErrorBody;

#[cfg(test)]
mod test;
