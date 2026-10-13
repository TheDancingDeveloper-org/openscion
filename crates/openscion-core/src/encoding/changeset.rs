//! The canonical changeset encoding, format version 1.

use openscion_proto::{Change, Changeset};

use super::tree::kind_byte;

const FORMAT_VERSION: u8 = 1;

/// Encodes a changeset canonically; its id is the hash of these bytes.
///
/// Integers are big-endian and every string is a `u32` byte length followed
/// by its UTF-8 bytes. Changes are written in the order given, which for a
/// stored [`Changeset`] is path order.
///
/// # Panics
///
/// Panics if a count or string length exceeds `u32::MAX`, which the engine's
/// change limit and the path rules rule out for any changeset it accepts.
#[must_use]
pub fn encode_changeset(changeset: &Changeset) -> Vec<u8> {
    let mut out = vec![FORMAT_VERSION];
    put_len(&mut out, changeset.parents.len());
    for parent in &changeset.parents {
        out.extend_from_slice(parent.as_bytes());
    }
    put_len(&mut out, changeset.changes.len());
    for change in &changeset.changes {
        match change {
            Change::Upsert { path, kind, blob } => {
                out.push(b'u');
                put_str(&mut out, path.as_str());
                out.push(kind_byte(*kind));
                out.extend_from_slice(blob.as_bytes());
            }
            Change::Delete { path } => {
                out.push(b'd');
                put_str(&mut out, path.as_str());
            }
        }
    }
    put_str(&mut out, &changeset.author);
    put_str(&mut out, &changeset.message);
    out.extend_from_slice(&changeset.created_at.to_be_bytes());
    put_len(&mut out, changeset.metadata.len());
    for (key, value) in &changeset.metadata {
        put_str(&mut out, key);
        put_str(&mut out, value);
    }
    out
}

fn put_len(out: &mut Vec<u8>, len: usize) {
    let len = u32::try_from(len).expect("lengths are bounded well below u32::MAX");
    out.extend_from_slice(&len.to_be_bytes());
}

fn put_str(out: &mut Vec<u8>, text: &str) {
    put_len(out, text.len());
    out.extend_from_slice(text.as_bytes());
}
