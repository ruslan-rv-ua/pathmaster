//! The Editing Session: a Working Copy over a Baseline (spec §5, ADR-0001).
//!
//! Two independent Sessions exist at runtime, one per Scope; each owns its
//! Working Copy (Entries + Value Type), its Baseline, its `writable` flag and
//! its Undo/Redo stacks. A Session never survives a process boundary — it is
//! plain owned state with no persistence.

use crate::path::split;

/// One of the two places Windows stores a `PATH`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Scope {
    User,
    System,
}

/// Whether a Scope's stored value expands `%VAR%` references (`REG_EXPAND_SZ`)
/// or holds them as literal text (`REG_SZ`). Part of the data: carried through
/// editing, compared for dirtiness, captured in every Checkpoint.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ValueType {
    RegSz,
    RegExpandSz,
}

/// A Scope's registry value as read: absent, or present with its type and raw
/// decoded string. Absent is distinct from present-and-empty (spec §4).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ScopeValue {
    Absent,
    Present { value_type: ValueType, raw: String },
}

/// Opaque Entry identity, unique within one Session, surviving Move and Edit.
/// Exists for focus restoration, never for the registry.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct EntryId(u64);

/// One `PATH` element: the raw substring between `;` separators, exactly as
/// read or as typed, plus its Session-local id.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Entry {
    id: EntryId,
    raw: String,
}

impl Entry {
    pub fn id(&self) -> EntryId {
        self.id
    }

    pub fn raw(&self) -> &str {
        &self.raw
    }
}

/// One entry in the undo history: a complete captured Working Copy (Entries
/// with ids, Value Type) plus the id of the Entry the change concerned, so
/// focus can return there (ADR-0001). Issues are a derived view and are
/// deliberately not part of it.
#[derive(Debug, Clone)]
struct Checkpoint {
    entries: Vec<Entry>,
    value_type: ValueType,
    focus: Option<EntryId>,
}

/// What an undo or redo did: where focus should go, if the operation
/// concerned a single Entry.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UndoOutcome {
    pub focus: Option<EntryId>,
}

/// One Scope's Editing Session.
#[derive(Debug)]
pub struct Session {
    scope: Scope,
    entries: Vec<Entry>,
    value_type: ValueType,
    // The Baseline keeps the Entries' ids so Cancel hands back the same
    // identities it discards from — dirtiness only ever compares the raws.
    baseline: Vec<Entry>,
    baseline_value_type: ValueType,
    writable: bool,
    undo: Vec<Checkpoint>,
    redo: Vec<Checkpoint>,
    next_id: u64,
}

impl Session {
    /// Loads a Session from a freshly read Scope value. Over an Absent Scope
    /// the Working Copy is empty and typed `REG_EXPAND_SZ` — the type the
    /// first Apply will create (spec §4).
    pub fn new(scope: Scope, value: ScopeValue, writable: bool) -> Self {
        let (raws, value_type) = decode(&value);
        let mut next_id = 0;
        let entries: Vec<Entry> = raws
            .into_iter()
            .map(|raw| fresh_entry(&mut next_id, raw))
            .collect();
        Session {
            scope,
            baseline: entries.clone(),
            entries,
            value_type,
            baseline_value_type: value_type,
            writable,
            undo: Vec::new(),
            redo: Vec::new(),
            next_id,
        }
    }

    pub fn scope(&self) -> Scope {
        self.scope
    }

    pub fn writable(&self) -> bool {
        self.writable
    }

    pub fn value_type(&self) -> ValueType {
        self.value_type
    }

    pub fn entries(&self) -> &[Entry] {
        &self.entries
    }

    /// Dirty is a comparison — order, raw strings, Value Type — never a flag.
    /// This one predicate drives Apply, Cancel, close-confirm and the
    /// Refresh/Restore warnings.
    pub fn is_dirty(&self) -> bool {
        self.value_type != self.baseline_value_type
            || self.entries.len() != self.baseline.len()
            || self
                .entries
                .iter()
                .zip(&self.baseline)
                .any(|(entry, baseline)| entry.raw != baseline.raw)
    }

    /// Appends a new Entry at the end (lowest search precedence).
    pub fn add(&mut self, raw: impl Into<String>) -> Option<EntryId> {
        if !self.writable {
            return None;
        }
        let entry = fresh_entry(&mut self.next_id, raw.into());
        let id = entry.id;
        self.push_checkpoint(Some(id));
        self.entries.push(entry);
        Some(id)
    }

    pub fn delete(&mut self, id: EntryId) -> bool {
        if !self.writable {
            return false;
        }
        let Some(index) = self.index_of(id) else {
            return false;
        };
        self.push_checkpoint(Some(id));
        self.entries.remove(index);
        true
    }

    /// Replaces an Entry's raw text; the id survives. A confirmed edit that
    /// changes nothing is not an operation.
    pub fn edit(&mut self, id: EntryId, raw: impl Into<String>) -> bool {
        if !self.writable {
            return false;
        }
        let Some(index) = self.index_of(id) else {
            return false;
        };
        let raw = raw.into();
        if self.entries[index].raw == raw {
            return false;
        }
        self.push_checkpoint(Some(id));
        self.entries[index].raw = raw;
        true
    }

    pub fn move_up(&mut self, id: EntryId) -> bool {
        if !self.writable {
            return false;
        }
        match self.index_of(id) {
            Some(index) if index > 0 => {
                self.push_checkpoint(Some(id));
                self.entries.swap(index, index - 1);
                true
            }
            _ => false,
        }
    }

    pub fn move_down(&mut self, id: EntryId) -> bool {
        if !self.writable {
            return false;
        }
        match self.index_of(id) {
            Some(index) if index + 1 < self.entries.len() => {
                self.push_checkpoint(Some(id));
                self.entries.swap(index, index + 1);
                true
            }
            _ => false,
        }
    }

    /// Changes the Working Copy's Value Type — only ever user-triggered,
    /// via the convert-or-keep dialog (spec §6). Same type is not a change.
    pub fn set_value_type(&mut self, value_type: ValueType) -> bool {
        if !self.writable || self.value_type == value_type {
            return false;
        }
        self.push_checkpoint(None);
        self.value_type = value_type;
        true
    }

    /// Records a successful Apply: the Baseline moves onto what was just
    /// written. Apply is a barrier, not a flush — the Undo/Redo stacks are
    /// never touched, so Ctrl+Z after Apply moves the Working Copy back and
    /// simply re-dirties the Session (spec §5).
    pub fn mark_applied(&mut self) {
        self.baseline = self.entries.clone();
        self.baseline_value_type = self.value_type;
    }

    /// Discards unsaved changes, returning the Working Copy to the Baseline.
    /// Cancel is itself a Checkpoint — Ctrl+Z restores the discarded work —
    /// and is disabled while the Session is clean.
    pub fn cancel(&mut self) -> bool {
        if !self.writable || !self.is_dirty() {
            return false;
        }
        self.push_checkpoint(None);
        self.entries = self.baseline.clone();
        self.value_type = self.baseline_value_type;
        true
    }

    /// Loads a Snapshot's decoded content into the Working Copy as one
    /// ordinary Checkpoint. Nothing reaches the registry until Apply.
    pub fn restore(&mut self, entries: Vec<String>, value_type: ValueType) -> bool {
        if !self.writable {
            return false;
        }
        self.push_checkpoint(None);
        self.entries = entries
            .into_iter()
            .map(|raw| fresh_entry(&mut self.next_id, raw))
            .collect();
        self.value_type = value_type;
        true
    }

    /// Replaces Working Copy and Baseline with a freshly read Scope value and
    /// clears both stacks — Checkpoints describe edits over a Baseline that no
    /// longer exists (ADR-0001). An Entry whose raw text survives the re-read
    /// keeps its id, so focus can stay on it (FR-refresh).
    pub fn refresh(&mut self, value: ScopeValue) {
        let (raws, value_type) = decode(&value);
        self.entries = self.entries_reusing_ids(&raws);
        self.value_type = value_type;
        self.baseline = self.entries.clone();
        self.baseline_value_type = value_type;
        self.undo.clear();
        self.redo.clear();
    }

    /// Runs several mutations as one user-visible operation — one Checkpoint,
    /// one undo step ("batches are one Checkpoint", FR-undo-redo). Ticket
    /// 11's convert-or-keep dialog commits an edit and a type change this
    /// way; v0.2.0's Fix Issues will batch multi-entry edits. A batch whose
    /// net effect is no change is not an operation and leaves no Checkpoint.
    pub fn batch(&mut self, focus: Option<EntryId>, mutate: impl FnOnce(&mut Session)) -> bool {
        if !self.writable {
            return false;
        }
        let before = self.capture(focus);
        let undo_depth = self.undo.len();
        let redo_before = std::mem::take(&mut self.redo);
        mutate(self);
        // The inner operations each pushed their own Checkpoint; the batch is one.
        self.undo.truncate(undo_depth);
        if self.entries == before.entries && self.value_type == before.value_type {
            self.redo = redo_before;
            return false;
        }
        self.undo.push(before);
        true
    }

    pub fn can_undo(&self) -> bool {
        !self.undo.is_empty()
    }

    pub fn can_redo(&self) -> bool {
        !self.redo.is_empty()
    }

    /// Restores the previous Checkpoint into the Working Copy. Never touches
    /// the registry or the Baseline — undoing past an Apply simply re-dirties
    /// the Session.
    pub fn undo(&mut self) -> Option<UndoOutcome> {
        if !self.writable {
            return None;
        }
        let checkpoint = self.undo.pop()?;
        self.redo.push(self.capture(checkpoint.focus));
        self.restore_checkpoint(checkpoint)
    }

    /// Reapplies the Checkpoint the last undo took back.
    pub fn redo(&mut self) -> Option<UndoOutcome> {
        if !self.writable {
            return None;
        }
        let checkpoint = self.redo.pop()?;
        self.undo.push(self.capture(checkpoint.focus));
        self.restore_checkpoint(checkpoint)
    }

    fn index_of(&self, id: EntryId) -> Option<usize> {
        self.entries.iter().position(|entry| entry.id == id)
    }

    /// Captures the current Working Copy as a Checkpoint.
    fn capture(&self, focus: Option<EntryId>) -> Checkpoint {
        Checkpoint {
            entries: self.entries.clone(),
            value_type: self.value_type,
            focus,
        }
    }

    /// One user-visible operation, one Checkpoint; a new operation truncates
    /// the Redo stack.
    fn push_checkpoint(&mut self, focus: Option<EntryId>) {
        let checkpoint = self.capture(focus);
        self.undo.push(checkpoint);
        self.redo.clear();
    }

    /// Builds Entries for `raws`, reusing the id of the first not-yet-matched
    /// current Entry with the same raw text; unmatched raws get fresh ids.
    fn entries_reusing_ids(&mut self, raws: &[String]) -> Vec<Entry> {
        let mut consumed = vec![false; self.entries.len()];
        let mut rehydrated = Vec::with_capacity(raws.len());
        for raw in raws {
            let reusable = self
                .entries
                .iter()
                .enumerate()
                .position(|(i, entry)| !consumed[i] && entry.raw == *raw);
            rehydrated.push(match reusable {
                Some(i) => {
                    consumed[i] = true;
                    Entry {
                        id: self.entries[i].id,
                        raw: raw.clone(),
                    }
                }
                None => fresh_entry(&mut self.next_id, raw.clone()),
            });
        }
        rehydrated
    }

    fn restore_checkpoint(&mut self, checkpoint: Checkpoint) -> Option<UndoOutcome> {
        let focus = self.landing_focus(&checkpoint);
        self.entries = checkpoint.entries;
        self.value_type = checkpoint.value_type;
        Some(UndoOutcome { focus })
    }

    /// Where focus lands after restoring `checkpoint`: the hinted Entry if
    /// the restored copy still holds it, else its nearest neighbour by index
    /// (the Entry at the hint's old position, clamped), else nothing.
    fn landing_focus(&self, checkpoint: &Checkpoint) -> Option<EntryId> {
        let hint = checkpoint.focus?;
        if checkpoint.entries.iter().any(|entry| entry.id == hint) {
            return Some(hint);
        }
        let index = self.index_of(hint)?;
        let last = checkpoint.entries.len().checked_sub(1)?;
        Some(checkpoint.entries[index.min(last)].id)
    }
}

fn decode(value: &ScopeValue) -> (Vec<String>, ValueType) {
    match value {
        ScopeValue::Absent => (Vec::new(), ValueType::RegExpandSz),
        ScopeValue::Present { value_type, raw } => (
            split(raw).into_iter().map(str::to_string).collect(),
            *value_type,
        ),
    }
}

fn fresh_entry(next_id: &mut u64, raw: String) -> Entry {
    let id = EntryId(*next_id);
    *next_id += 1;
    Entry { id, raw }
}
