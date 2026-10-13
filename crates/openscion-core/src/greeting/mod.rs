//! Greeting behavior used to demonstrate the crate.s module layout until phase 1 replaces it.
//!
//! A module root like this one documents the module, wires its pieces
//! together, and exposes the smallest useful API. Substantial type definitions
//! belong in a sibling `types.rs`, and unit tests belong in `test.rs`, wired in
//! at the bottom of this file.
//!
//! Phase 1 replaces this module with the core data model.

use crate::{Error, Result};

/// Returns a friendly greeting for `name`.
///
/// Surrounding whitespace is trimmed before the greeting is built.
///
/// # Examples
///
/// ```
/// # use openscion_core::greet;
/// assert_eq!(greet("  Ferris  ")?, "Hello, Ferris!");
/// # Ok::<(), openscion_core::Error>(())
/// ```
///
/// # Errors
///
/// Returns [`Error::EmptyName`] when `name` is empty or contains only
/// whitespace.
pub fn greet(name: &str) -> Result<String> {
    let name = name.trim();
    if name.is_empty() {
        return Err(Error::EmptyName);
    }

    Ok(format!("Hello, {name}!"))
}

#[cfg(test)]
mod test;
