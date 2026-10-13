//! The validated repository path type.

use std::fmt;

use serde::{Deserialize, Deserializer, Serialize, Serializer};

/// A valid path to a file, relative to the repository root.
///
/// # Examples
///
/// ```
/// # use openscion_proto::RepoPath;
/// let path = RepoPath::new("src/lib.rs")?;
/// assert_eq!(path.components().collect::<Vec<_>>(), ["src", "lib.rs"]);
/// assert!(RepoPath::new("src/../etc").is_err());
/// # Ok::<(), openscion_proto::PathError>(())
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct RepoPath(String);

impl RepoPath {
    /// The longest a single component may be, in bytes.
    pub const MAX_COMPONENT: usize = 255;
    /// The longest a whole path may be, in bytes.
    pub const MAX_LEN: usize = 4096;

    /// Validates `path` and wraps it.
    ///
    /// # Errors
    ///
    /// Returns a [`PathError`] naming the path and the rule it breaks.
    pub fn new(path: impl Into<String>) -> Result<Self, PathError> {
        let path = path.into();
        match violation(&path) {
            None => Ok(Self(path)),
            Some(rule) => Err(PathError { path, rule }),
        }
    }

    /// The path as a string.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// The path's components, root first.
    pub fn components(&self) -> impl Iterator<Item = &str> {
        self.0.split('/')
    }
}

/// Checks a single path component, returning the rule it breaks.
pub(crate) fn component_violation(component: &str) -> Option<&'static str> {
    if component.is_empty() {
        Some("empty component")
    } else if component == "." || component == ".." {
        Some("`.` or `..` component")
    } else if component.contains('\0') {
        Some("component contains NUL")
    } else if component.contains('/') {
        Some("component contains `/`")
    } else if component.len() > RepoPath::MAX_COMPONENT {
        Some("component longer than 255 bytes")
    } else {
        None
    }
}

fn violation(path: &str) -> Option<&'static str> {
    if path.len() > RepoPath::MAX_LEN {
        return Some("path longer than 4096 bytes");
    }
    path.split('/').find_map(component_violation)
}

impl fmt::Display for RepoPath {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl Serialize for RepoPath {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.0)
    }
}

impl<'de> Deserialize<'de> for RepoPath {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        Self::new(String::deserialize(deserializer)?).map_err(serde::de::Error::custom)
    }
}

/// A path that breaks one of the path rules.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct PathError {
    /// The rejected path.
    pub path: String,
    /// The rule it breaks.
    pub rule: &'static str,
}

impl fmt::Display for PathError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "invalid path {:?}: {}", self.path, self.rule)
    }
}

impl std::error::Error for PathError {}
