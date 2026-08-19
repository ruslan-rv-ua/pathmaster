# 02 — Core editing model

**Spec:** [spec §5](../../pathmaster-v0-1-0/spec.md) · ADR-0001, ADR-0007

**What to build:** The pure editing model in `pathmaster-core`, fully verifiable by `cargo test` on any OS (per ADR-0007 the crate boundary is the test boundary — this is the complete slice). Splitting a raw PATH value into Entries, an Editing Session with Working Copy, Baseline, dirty-as-comparison, and a Checkpoint-based Undo/Redo stack with Apply as a barrier.

**Blocked by:** None — can start immediately.

**Status:** ready-for-agent

- [ ] Split/join: an Entry is the raw substring between `;` separators, byte-for-byte; split-then-join reproduces the decoded value exactly; an empty value decodes to zero Entries, not one empty Entry
- [ ] Each Entry carries an opaque id surviving Move and Edit (for focus restoration)
- [ ] Session holds Working Copy (Entries + Value Type), Baseline, `writable` flag, Undo/Redo stacks; two independent Sessions, one per Scope; a Session never survives a process boundary
- [ ] Dirty is a comparison (order, raw strings, Value Type) — never a flag; one predicate; an edit and its exact reversal leave the Session clean
- [ ] Checkpoints are whole-copy captures (Entries with ids, Value Type, focus hint), exactly one per user-visible operation (Add, Delete, Move Up, Move Down, confirmed Edit, type change, Cancel, Restore); a new operation truncates the Redo stack
- [ ] Apply is a barrier, not a flush: undo past it moves the Working Copy only and re-dirties the Session; moving the Baseline never touches the stacks
- [ ] Cancel is itself a Checkpoint (the discarded work is restorable by undo); Refresh clears both stacks
- [ ] Property test (proptest, dev-dependency of core alone): split→join byte-identity
