//! The canonical tree encoding.

use openscion_proto::{EntryKind, ObjectId, RepoPath, TreeEntry};

use crate::{Error, Result};

const WHAT: &str = "tree";

pub(crate) fn kind_byte(kind: EntryKind) -> u8 {
    match kind {
        EntryKind::File => b'f',
        EntryKind::Executable => b'x',
        EntryKind::Symlink => b'l',
        EntryKind::Tree => b'd',
    }
}

fn kind_from_byte(byte: u8) -> Result<EntryKind> {
    match byte {
        b'f' => Ok(EntryKind::File),
        b'x' => Ok(EntryKind::Executable),
        b'l' => Ok(EntryKind::Symlink),
        b'd' => Ok(EntryKind::Tree),
        _ => Err(malformed("unknown entry kind")),
    }
}

fn malformed(reason: &'static str) -> Error {
    Error::Malformed { what: WHAT, reason }
}

/// Encodes `entries` canonically. The entries must already be sorted by name
/// with unique, valid names, as the engine's own trees always are.
///
/// # Examples
///
/// ```
/// use openscion_core::{EntryKind, ObjectId, TreeEntry, decode_tree, encode_tree};
///
/// let entries = vec![TreeEntry::new("a.txt", EntryKind::File, ObjectId::from_bytes([7; 32]))];
/// let bytes = encode_tree(&entries);
/// assert_eq!(bytes.len(), 1 + 5 + 1 + 32);
/// assert_eq!(decode_tree(&bytes)?, entries);
/// # Ok::<(), openscion_core::Error>(())
/// ```
#[must_use]
pub fn encode_tree(entries: &[TreeEntry]) -> Vec<u8> {
    let size = entries
        .iter()
        .map(|entry| entry.name.len() + 2 + ObjectId::LEN)
        .sum();
    let mut out = Vec::with_capacity(size);
    for entry in entries {
        out.push(kind_byte(entry.kind));
        out.extend_from_slice(entry.name.as_bytes());
        out.push(0);
        out.extend_from_slice(entry.id.as_bytes());
    }
    out
}

/// Decodes a canonical tree encoding.
///
/// # Errors
///
/// Returns [`Error::Malformed`] when the bytes are truncated, hold an unknown
/// kind byte or an invalid name, or list names out of order or twice.
pub fn decode_tree(mut bytes: &[u8]) -> Result<Vec<TreeEntry>> {
    let mut entries: Vec<TreeEntry> = Vec::new();
    while let Some((&kind, rest)) = bytes.split_first() {
        let kind = kind_from_byte(kind)?;
        let nul = rest
            .iter()
            .position(|&byte| byte == 0)
            .ok_or_else(|| malformed("unterminated name"))?;
        let name =
            std::str::from_utf8(&rest[..nul]).map_err(|_| malformed("name is not UTF-8"))?;
        if name.contains('/') || RepoPath::new(name).is_err() {
            return Err(malformed("invalid entry name"));
        }
        if entries.last().is_some_and(|last| last.name.as_str() >= name) {
            return Err(malformed("entries out of order"));
        }
        let rest = &rest[nul + 1..];
        let id: [u8; ObjectId::LEN] = rest
            .get(..ObjectId::LEN)
            .and_then(|slice| slice.try_into().ok())
            .ok_or_else(|| malformed("truncated id"))?;
        entries.push(TreeEntry::new(name, kind, ObjectId::from_bytes(id)));
        bytes = &rest[ObjectId::LEN..];
    }
    Ok(entries)
}
