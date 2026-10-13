//! The object store: blobs and trees by id.
//!
//! [`MemoryObjectStore`] keeps every object in memory. It is the only store
//! the MVP has; a persistent store implements the same operations behind the
//! same port in a later phase (`docs/specs/openscion-architecture.md`).
//! Objects are immutable and content-addressed, so a put of an object that is
//! already present is a no-op.

use std::collections::HashMap;
use std::sync::Arc;

use openscion_proto::{ObjectId, ObjectKind};

use crate::{Error, Result, object_id};

/// Every object the server holds, in memory.
#[derive(Debug, Default)]
pub struct MemoryObjectStore {
    objects: HashMap<ObjectId, Arc<[u8]>>,
    bytes: u64,
}

impl MemoryObjectStore {
    /// An empty store.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Hashes `payload` as an object of `kind`, stores it, and returns its id.
    pub fn put(&mut self, kind: ObjectKind, payload: &[u8]) -> ObjectId {
        let id = object_id(kind, payload);
        self.insert(id, payload);
        id
    }

    /// Stores `payload` under `expected` after checking that it hashes to it.
    ///
    /// # Errors
    ///
    /// Returns [`Error::CorruptObject`] when the payload hashes to a different
    /// id; nothing is stored.
    pub fn put_verified(
        &mut self,
        kind: ObjectKind,
        expected: ObjectId,
        payload: &[u8],
    ) -> Result<()> {
        let actual = object_id(kind, payload);
        if actual != expected {
            return Err(Error::CorruptObject { expected, actual });
        }
        self.insert(actual, payload);
        Ok(())
    }

    fn insert(&mut self, id: ObjectId, payload: &[u8]) {
        self.objects.entry(id).or_insert_with(|| {
            self.bytes += payload.len() as u64;
            Arc::from(payload)
        });
    }

    /// The object's bytes.
    ///
    /// # Errors
    ///
    /// Returns [`Error::NotFound`] when no object has this id.
    pub fn get(&self, id: ObjectId) -> Result<Arc<[u8]>> {
        self.objects.get(&id).cloned().ok_or(Error::NotFound(id))
    }

    /// Whether an object with this id is stored.
    #[must_use]
    pub fn contains(&self, id: ObjectId) -> bool {
        self.objects.contains_key(&id)
    }

    /// The number of distinct objects stored.
    #[must_use]
    pub fn len(&self) -> usize {
        self.objects.len()
    }

    /// Whether the store holds nothing.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.objects.is_empty()
    }

    /// The total size of every stored payload, in bytes.
    #[must_use]
    pub fn total_bytes(&self) -> u64 {
        self.bytes
    }
}

#[cfg(test)]
mod test;
