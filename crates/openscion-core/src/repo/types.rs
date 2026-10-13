//! The log entry type.

use openscion_proto::{Changeset, ChangesetInfo};

/// One appended changeset and its summary.
#[derive(Debug, Clone)]
pub(super) struct LogEntry {
    pub(super) info: ChangesetInfo,
    pub(super) changeset: Changeset,
}
