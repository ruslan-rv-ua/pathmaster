//! The backup budget: how many Snapshots of a Scope are kept, and which ones
//! go when there are more (spec §8, FR-backup-rotation).
//!
//! Two decisions live here and nowhere else. `maxBackups` is an **independent
//! per-Scope budget** — never a single count pooled across the directory,
//! which would let fifty User Applies silently wipe every System Snapshot.
//! And rotation reads **file names only**: the Scope and the age of every file
//! are in its name, so a Corrupted Snapshot is indistinguishable from a good
//! one here and rotates exactly like it.
//!
//! Deleting is the caller's, in the imperative shell. This module names files;
//! it never touches one.

use crate::session::Scope;
use crate::snapshot::SnapshotName;

/// The budget's floor. `settings.json` already rejects anything below it
/// (spec §13), so this is not a second reading of the file — it is the floor
/// under a caller's mistake at the one step that deletes: a zero budget would
/// delete the Snapshot the Apply in progress has just taken.
const AT_LEAST: u32 = 1;

/// The Snapshots of `scope` that no longer fit its budget, oldest first —
/// what the caller is to delete.
///
/// `listing` is the whole directory, both Scopes and any number of Snapshots;
/// the other Scope's files are never named, however old they are. A file
/// another instance has already deleted is simply not in the listing it was
/// given, and one that goes between here and the delete is a not-found the
/// caller treats as success — nothing in the selection depends on a file
/// surviving until it is deleted.
pub fn overflow(listing: &[SnapshotName], scope: Scope, max_backups: u32) -> Vec<&SnapshotName> {
    let mut of_scope: Vec<&SnapshotName> = listing
        .iter()
        .filter(|name| name.scope() == scope)
        .collect();
    // Ordered here rather than assumed of the caller: "the oldest is deleted"
    // is a rule about ages, and it should not quietly become a rule about the
    // order a Vec happened to arrive in.
    of_scope.sort();
    let keep = max_backups.max(AT_LEAST) as usize;
    of_scope.truncate(of_scope.len().saturating_sub(keep));
    of_scope
}
