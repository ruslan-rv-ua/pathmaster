# 02 — Core editing model

**Spec:** [spec §5](../../pathmaster-v0-1-0/spec.md) · ADR-0001, ADR-0007

**What to build:** The pure editing model in `pathmaster-core`, fully verifiable by `cargo test` on any OS (per ADR-0007 the crate boundary is the test boundary — this is the complete slice). Splitting a raw PATH value into Entries, an Editing Session with Working Copy, Baseline, dirty-as-comparison, and a Checkpoint-based Undo/Redo stack with Apply as a barrier.

**Blocked by:** None — can start immediately.

**Status:** resolved

- [x] Split/join: an Entry is the raw substring between `;` separators, byte-for-byte; split-then-join reproduces the decoded value exactly; an empty value decodes to zero Entries, not one empty Entry
- [x] Each Entry carries an opaque id surviving Move and Edit (for focus restoration)
- [x] Session holds Working Copy (Entries + Value Type), Baseline, `writable` flag, Undo/Redo stacks; two independent Sessions, one per Scope; a Session never survives a process boundary
- [x] Dirty is a comparison (order, raw strings, Value Type) — never a flag; one predicate; an edit and its exact reversal leave the Session clean
- [x] Checkpoints are whole-copy captures (Entries with ids, Value Type, focus hint), exactly one per user-visible operation (Add, Delete, Move Up, Move Down, confirmed Edit, type change, Cancel, Restore); a new operation truncates the Redo stack
- [x] Apply is a barrier, not a flush: undo past it moves the Working Copy only and re-dirties the Session; moving the Baseline never touches the stacks
- [x] Cancel is itself a Checkpoint (the discarded work is restorable by undo); Refresh clears both stacks
- [x] Property test (proptest, dev-dependency of core alone): split→join byte-identity

## Comments

Implemented 2026-08-19, TDD at the crate boundary (ADR-0007): 35 integration tests in
`crates/pathmaster-core/tests/{path,session}.rs`, no test links wx.

- **`path` module**: `split` / `join`, byte-for-byte on `str::split(';')`; empty value → zero
  Entries; the round trip holds for trailing `;`, `;;`, and lone `;` — plus the proptest
  byte-identity property over arbitrary strings (proptest is a dev-dependency of core alone).
- **`session` module**: `Scope`, `ValueType { RegSz, RegExpandSz }`,
  `ScopeValue { Absent, Present }`, `Entry` with opaque `EntryId`, and `Session`. An Absent
  Scope loads as zero Entries typed `REG_EXPAND_SZ` and clean (issue 06 §4).
- **Dirty** is one predicate (`is_dirty`): order + raw strings + Value Type against the
  Baseline. Add-then-Delete, Move Down-then-Up, edit-and-reverse, type-change-and-back all
  verified clean.
- **Checkpoints** capture the whole pre-operation Working Copy plus a focus hint (the id of
  the Entry the change concerned; `None` for type change / Cancel / Restore). Undo/redo
  resolve the hint against the restored copy: if the hinted Entry is gone (undo of an Add,
  redo of a Delete), the outcome hints the nearest neighbour by index instead — never a
  dangling id. No-ops (move at an edge, edit with unchanged text, same-type set) refuse and
  leave no Checkpoint, so Ctrl+Z is never a visible nothing.
- **`batch`** runs several mutations as one user-visible operation — one Checkpoint, one
  undo step ("batches are one Checkpoint", FR-undo-redo). Ticket 11's convert-or-keep dialog
  commits edit + type change through it; v0.2.0's Fix Issues gets multi-entry batches free.
  A batch whose net effect is no change leaves no Checkpoint.
- **Apply is `mark_applied`**: moves the Baseline onto the Working Copy, touches neither
  stack. The registry write itself is platform's (ticket 03/13); core only records that it
  succeeded.
- **Refresh** takes the freshly read `ScopeValue`, resets both copies, clears both stacks,
  and preserves the id of any Entry whose raw text survives the re-read (first-match) so
  FR-refresh's focus rule has something to stand on.
- **`writable` is enforced in core**, not just stored: every mutating method (including
  undo/redo) refuses on a non-writable Session — defence in depth under the UI's disabling.
- **The Baseline stores Entries with their ids** (dirtiness compares only the raws), so
  Cancel hands back the same identities it discards from — an Entry edited then cancelled
  keeps its id for focus restoration.

Two-axis review (Standards / Spec) run before commit. Spec axis found the missing
batch-as-one-Checkpoint mechanism and the dangling focus hint on undo-of-Add; both fixed
test-first (the three review-fix tests above). Standards axis: clean apart from noted
judgement calls (undo/redo mirror shape, `entries`+`value_type` travelling unbundled) —
left as-is deliberately; and the standing precedent of committing impl tickets directly to
`develop` despite AGENTS.md's git-flow preference.
