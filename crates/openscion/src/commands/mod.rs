//! The working-copy commands: `import`, `commit`, and `clone`.
//!
//! Each command talks to the server through a [`ServerClient`] and to the
//! disk through the working-copy module, and returns a summary the CLI
//! prints.
//!
//! - **import** records a whole directory as the repository's first
//!   changeset and creates the ref.
//! - **commit** records what changed since the base changeset and moves the
//!   ref from the base to the new changeset by compare-and-swap. If someone
//!   else moved it first the swap fails; the MVP has no rebase, so the
//!   changeset stays in the log unreferenced and the error is reported.
//! - **clone** reads the ref, fetches the trees breadth first — one request
//!   per level — and then the blobs in batches, verifying every id. With a
//!   path it fetches only the trees on the way to that directory and the
//!   files under it.
//!
//! [`ServerClient`]: crate::ServerClient

mod clone;
mod commit;
mod import;

pub(crate) use clone::clone;
pub(crate) use commit::commit;
pub(crate) use import::import;

use openscion_core::{FrameWriter, ObjectId};

use crate::{Result, ServerClient};

/// Upload batches are cut at this many bytes of blob data.
const UPLOAD_BATCH_BYTES: usize = 64 << 20;
/// Fetch requests name at most this many objects.
const FETCH_BATCH: usize = 4096;

/// Uploads blobs in batches of at most [`UPLOAD_BATCH_BYTES`].
fn upload(client: &ServerClient, blobs: &[(ObjectId, Vec<u8>)]) -> Result<()> {
    let mut frames = FrameWriter::new();
    for (id, bytes) in blobs {
        if !frames.is_empty() && frames.len() + bytes.len() > UPLOAD_BATCH_BYTES {
            client.upload_blobs(&std::mem::take(&mut frames).finish())?;
        }
        frames.push(*id, bytes);
    }
    if !frames.is_empty() {
        client.upload_blobs(&frames.finish())?;
    }
    Ok(())
}

#[cfg(test)]
mod test;
