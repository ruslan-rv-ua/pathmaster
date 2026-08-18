# Undo is a stack of whole-Working-Copy Checkpoints, not invertible commands

`PATH` editing needs unbounded undo within a session (FR-undo-redo), and v0.2.0's Fix Issues must undo a
multi-entry batch as a single step. The obvious shape — a command per operation, each knowing how to invert
itself — was rejected in favour of pushing a **complete copy of the Working Copy** onto the stack for every
user-visible operation. A `PATH` is bounded at roughly 200 Entries, so one Checkpoint costs a few kilobytes
and even a thousand-step history stays in single-digit megabytes; against that, inversion bugs are a whole
class of defect that silently corrupts the very data the application exists to protect, and batching comes
free rather than needing a composite-command mechanism.

## Consequences

- **The Value Type is undone along with the Entries**, because it is part of the Working Copy. A `REG_SZ` →
  `REG_EXPAND_SZ` conversion is therefore reverted by Ctrl+Z like any other edit, with no special handling.
- **Cancel is itself a Checkpoint**, so discarding unsaved changes is recoverable with Ctrl+Z. Under a command
  model this would have needed its own inverse; here it is just another captured state.
- **Refresh clears the stack**, and must. Checkpoints describe edits over a Baseline that no longer exists, so
  undoing past a Refresh would re-arm a working copy that overwrites an external change without the
  external-modification dialog ever firing again.
- **Issues are excluded from the Checkpoint** and recomputed after a restore. Capturing them would let Undo
  reinstate a diagnosis of a state that is no longer displayed.
