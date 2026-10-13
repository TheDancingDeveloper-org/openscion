//! The frame stream that carries batches of objects.

use openscion_proto::ObjectId;

use crate::{Error, Result};

const WHAT: &str = "frame";
const HEADER: usize = ObjectId::LEN + 8;

/// Builds a frame stream: one `id ‖ u64 length ‖ bytes` frame per object.
///
/// # Examples
///
/// ```
/// use openscion_core::{FrameWriter, ObjectId, decode_frames};
///
/// let id = ObjectId::from_bytes([1; 32]);
/// let mut frames = FrameWriter::new();
/// frames.push(id, b"hello");
/// let bytes = frames.finish();
/// assert_eq!(decode_frames(&bytes)?, vec![(id, &b"hello"[..])]);
/// # Ok::<(), openscion_core::Error>(())
/// ```
#[derive(Debug, Default)]
pub struct FrameWriter {
    out: Vec<u8>,
}

impl FrameWriter {
    /// Starts an empty stream.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Appends one object.
    pub fn push(&mut self, id: ObjectId, bytes: &[u8]) {
        self.out.reserve(HEADER + bytes.len());
        self.out.extend_from_slice(id.as_bytes());
        self.out.extend_from_slice(&(bytes.len() as u64).to_be_bytes());
        self.out.extend_from_slice(bytes);
    }

    /// The number of bytes written so far.
    #[must_use]
    pub fn len(&self) -> usize {
        self.out.len()
    }

    /// Whether nothing has been written.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.out.is_empty()
    }

    /// Returns the finished stream.
    #[must_use]
    pub fn finish(self) -> Vec<u8> {
        self.out
    }
}

/// Splits a frame stream into `(id, bytes)` pairs, borrowing from `bytes`.
///
/// The ids are taken as written; callers that need to trust them verify each
/// one against its bytes.
///
/// # Errors
///
/// Returns [`Error::Malformed`] when a frame's header or body is truncated.
pub fn decode_frames(mut bytes: &[u8]) -> Result<Vec<(ObjectId, &[u8])>> {
    let mut frames = Vec::new();
    while !bytes.is_empty() {
        let (id, rest) = bytes
            .split_first_chunk::<{ ObjectId::LEN }>()
            .ok_or_else(|| truncated("truncated id"))?;
        let (len, rest) = rest
            .split_first_chunk::<8>()
            .ok_or_else(|| truncated("truncated length"))?;
        let len = usize::try_from(u64::from_be_bytes(*len))
            .map_err(|_| truncated("length out of range"))?;
        if rest.len() < len {
            return Err(truncated("truncated body"));
        }
        let (body, rest) = rest.split_at(len);
        frames.push((ObjectId::from_bytes(*id), body));
        bytes = rest;
    }
    Ok(frames)
}

fn truncated(reason: &'static str) -> Error {
    Error::Malformed { what: WHAT, reason }
}
