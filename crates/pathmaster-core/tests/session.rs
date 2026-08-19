//! The Editing Session at the crate boundary (spec §5, ADR-0001, ticket impl-02).

use pathmaster_core::session::{Scope, ScopeValue, Session, ValueType};

fn present(raw: &str) -> ScopeValue {
    ScopeValue::Present {
        value_type: ValueType::RegExpandSz,
        raw: raw.to_string(),
    }
}

fn user_session(raw: &str) -> Session {
    Session::new(Scope::User, present(raw), true)
}

fn raws(session: &Session) -> Vec<&str> {
    session.entries().iter().map(|e| e.raw()).collect()
}

// ---- Construction ----

#[test]
fn a_freshly_loaded_session_holds_the_value_and_is_clean() {
    let session = user_session(r"C:\one;C:\two");
    assert_eq!(raws(&session), vec![r"C:\one", r"C:\two"]);
    assert_eq!(session.value_type(), ValueType::RegExpandSz);
    assert!(session.writable());
    assert!(!session.is_dirty());
}

#[test]
fn an_absent_scope_loads_as_zero_entries_reg_expand_sz_and_clean() {
    let session = Session::new(Scope::System, ScopeValue::Absent, true);
    assert!(session.entries().is_empty());
    assert_eq!(session.value_type(), ValueType::RegExpandSz);
    assert!(!session.is_dirty());
}

#[test]
fn entry_ids_are_unique_within_a_session() {
    let session = user_session(r"C:\one;C:\two;C:\one");
    let mut ids: Vec<_> = session.entries().iter().map(|e| e.id()).collect();
    ids.sort();
    ids.dedup();
    assert_eq!(ids.len(), 3);
}

// ---- Dirty is a comparison, never a flag ----

#[test]
fn an_edit_makes_the_session_dirty() {
    let mut session = user_session(r"C:\one;C:\two");
    let id = session.entries()[0].id();
    assert!(session.edit(id, r"C:\changed"));
    assert!(session.is_dirty());
}

#[test]
fn an_edit_and_its_exact_reversal_leave_the_session_clean() {
    let mut session = user_session(r"C:\one;C:\two");
    let id = session.entries()[0].id();
    session.edit(id, r"C:\changed");
    session.edit(id, r"C:\one");
    assert!(!session.is_dirty());
}

#[test]
fn add_then_delete_leaves_the_session_clean() {
    let mut session = user_session(r"C:\one");
    let id = session.add(r"C:\new").expect("writable session accepts Add");
    assert!(session.is_dirty());
    assert!(session.delete(id));
    assert!(!session.is_dirty());
}

#[test]
fn move_down_then_move_up_leaves_the_session_clean() {
    let mut session = user_session(r"C:\one;C:\two");
    let id = session.entries()[0].id();
    assert!(session.move_down(id));
    assert_eq!(raws(&session), vec![r"C:\two", r"C:\one"]);
    assert!(session.is_dirty());
    assert!(session.move_up(id));
    assert!(!session.is_dirty());
}

// ---- Checkpoints: one per user-visible operation (ADR-0001) ----

#[test]
fn undo_on_a_fresh_session_has_nothing_to_do() {
    let mut session = user_session(r"C:\one");
    assert!(!session.can_undo());
    assert!(session.undo().is_none());
}

#[test]
fn each_operation_is_exactly_one_undo_step() {
    let mut session = user_session(r"C:\one;C:\two");
    let first = session.entries()[0].id();
    session.add(r"C:\three");
    session.move_down(first);
    session.edit(first, r"C:\edited");

    assert!(session.undo().is_some());
    assert_eq!(raws(&session), vec![r"C:\two", r"C:\one", r"C:\three"]);
    assert!(session.undo().is_some());
    assert_eq!(raws(&session), vec![r"C:\one", r"C:\two", r"C:\three"]);
    assert!(session.undo().is_some());
    assert_eq!(raws(&session), vec![r"C:\one", r"C:\two"]);
    assert!(!session.is_dirty());
    assert!(session.undo().is_none());
}

#[test]
fn redo_reapplies_what_undo_took_back() {
    let mut session = user_session(r"C:\one");
    session.add(r"C:\two");
    session.undo();
    assert_eq!(raws(&session), vec![r"C:\one"]);
    assert!(session.redo().is_some());
    assert_eq!(raws(&session), vec![r"C:\one", r"C:\two"]);
    assert!(session.redo().is_none());
}

#[test]
fn a_new_operation_truncates_the_redo_stack() {
    let mut session = user_session(r"C:\one");
    session.add(r"C:\two");
    session.undo();
    assert!(session.can_redo());
    session.add(r"C:\other");
    assert!(!session.can_redo());
    assert!(session.redo().is_none());
}

#[test]
fn undo_restores_the_value_type_along_with_the_entries() {
    let mut session = user_session(r"C:\one");
    session.set_value_type(ValueType::RegSz);
    session.undo();
    assert_eq!(session.value_type(), ValueType::RegExpandSz);
    session.redo();
    assert_eq!(session.value_type(), ValueType::RegSz);
}

#[test]
fn undo_and_redo_hint_the_entry_the_change_concerned() {
    let mut session = user_session(r"C:\one");
    let id = session.entries()[0].id();
    session.edit(id, r"C:\edited");
    assert_eq!(session.undo().expect("one step to undo").focus, Some(id));
    assert_eq!(session.redo().expect("one step to redo").focus, Some(id));
}

#[test]
fn entry_ids_survive_undo_and_redo() {
    let mut session = user_session(r"C:\one;C:\two");
    let first = session.entries()[0].id();
    session.move_down(first);
    session.undo();
    assert_eq!(session.entries()[0].id(), first);
    session.redo();
    assert_eq!(session.entries()[1].id(), first);
}

#[test]
fn a_rejected_no_op_leaves_no_checkpoint() {
    let mut session = user_session(r"C:\one;C:\two");
    let first = session.entries()[0].id();
    let last = session.entries()[1].id();
    assert!(!session.move_up(first));
    assert!(!session.move_down(last));
    assert!(!session.edit(first, r"C:\one"));
    assert!(!session.set_value_type(ValueType::RegExpandSz));
    assert!(!session.can_undo());
}

#[test]
fn a_value_type_change_alone_makes_the_session_dirty() {
    let mut session = user_session(r"C:\one");
    assert!(session.set_value_type(ValueType::RegSz));
    assert!(session.is_dirty());
    assert!(session.set_value_type(ValueType::RegExpandSz));
    assert!(!session.is_dirty());
}

#[test]
fn a_batch_is_one_checkpoint_however_much_it_touches() {
    // Ticket 11's convert-or-keep dialog commits an edit and a type change
    // as one user-visible operation (FR-edit-f2, FR-undo-redo: "batches are
    // one Checkpoint").
    let mut session = user_session(r"C:\one;C:\two");
    let id = session.entries()[0].id();
    assert!(session.batch(Some(id), |s| {
        s.edit(id, r"C:\%VAR%");
        s.set_value_type(ValueType::RegSz);
    }));

    assert_eq!(session.undo().expect("one step to undo").focus, Some(id));
    assert_eq!(raws(&session), vec![r"C:\one", r"C:\two"]);
    assert_eq!(session.value_type(), ValueType::RegExpandSz);
    assert!(!session.can_undo(), "the whole batch was a single Checkpoint");

    assert!(session.redo().is_some());
    assert_eq!(raws(&session), vec![r"C:\%VAR%", r"C:\two"]);
    assert_eq!(session.value_type(), ValueType::RegSz);
}

#[test]
fn a_batch_that_changes_nothing_is_not_an_operation() {
    let mut session = user_session(r"C:\one");
    let id = session.entries()[0].id();
    assert!(!session.batch(Some(id), |s| {
        s.edit(id, r"C:\other");
        s.edit(id, r"C:\one");
    }));
    assert!(!session.can_undo());
    assert!(!session.is_dirty());
}

#[test]
fn undoing_an_add_hints_the_nearest_surviving_neighbour() {
    let mut session = user_session(r"C:\one");
    let survivor = session.entries()[0].id();
    session.add(r"C:\added");
    // The added Entry does not exist in the restored copy; the hint falls
    // back to the Entry at the same clamped index.
    assert_eq!(session.undo().expect("one step to undo").focus, Some(survivor));
}

#[test]
fn undoing_the_only_add_in_an_empty_scope_hints_nothing() {
    let mut session = Session::new(Scope::User, ScopeValue::Absent, true);
    session.add(r"C:\added");
    assert_eq!(session.undo().expect("one step to undo").focus, None);
}

// ---- Apply is a barrier, not a flush ----

#[test]
fn apply_moves_the_baseline_and_leaves_both_stacks_untouched() {
    let mut session = user_session(r"C:\one");
    session.add(r"C:\two");
    session.undo();
    session.redo();
    session.mark_applied();
    assert!(!session.is_dirty());
    assert!(session.can_undo());
}

#[test]
fn undo_past_apply_moves_the_working_copy_only_and_re_dirties() {
    let mut session = user_session(r"C:\one");
    session.add(r"C:\two");
    session.mark_applied();
    assert!(session.undo().is_some());
    assert_eq!(raws(&session), vec![r"C:\one"]);
    assert!(session.is_dirty(), "the Baseline stayed at the applied value");
    session.redo();
    assert!(!session.is_dirty());
}

// ---- Cancel is itself a Checkpoint ----

#[test]
fn cancel_returns_the_working_copy_to_the_baseline() {
    let mut session = user_session(r"C:\one;C:\two");
    let id = session.entries()[0].id();
    session.edit(id, r"C:\edited");
    session.set_value_type(ValueType::RegSz);
    assert!(session.cancel());
    assert_eq!(raws(&session), vec![r"C:\one", r"C:\two"]);
    assert_eq!(session.value_type(), ValueType::RegExpandSz);
    assert!(!session.is_dirty());
}

#[test]
fn cancel_is_disabled_while_clean() {
    let mut session = user_session(r"C:\one");
    assert!(!session.cancel());
    assert!(!session.can_undo());
}

#[test]
fn an_entry_edited_then_cancelled_keeps_its_id() {
    let mut session = user_session(r"C:\one;C:\two");
    let id = session.entries()[0].id();
    session.edit(id, r"C:\edited");
    session.cancel();
    assert_eq!(session.entries()[0].id(), id);
}

#[test]
fn undo_after_cancel_restores_the_discarded_work() {
    let mut session = user_session(r"C:\one");
    session.add(r"C:\two");
    session.cancel();
    assert!(session.undo().is_some());
    assert_eq!(raws(&session), vec![r"C:\one", r"C:\two"]);
    assert!(session.is_dirty());
}

// ---- Restore is one ordinary Checkpoint ----

#[test]
fn restore_loads_a_snapshot_as_one_undoable_checkpoint() {
    let mut session = user_session(r"C:\one;C:\two");
    assert!(session.restore(
        vec![r"D:\restored".to_string()],
        ValueType::RegSz,
    ));
    assert_eq!(raws(&session), vec![r"D:\restored"]);
    assert_eq!(session.value_type(), ValueType::RegSz);
    assert!(session.is_dirty(), "Restore edits the Working Copy, never the registry");
    session.undo();
    assert_eq!(raws(&session), vec![r"C:\one", r"C:\two"]);
    assert!(!session.is_dirty());
}

// ---- Refresh ----

#[test]
fn refresh_resets_working_copy_and_baseline_and_clears_both_stacks() {
    let mut session = user_session(r"C:\one");
    session.add(r"C:\two");
    session.undo();
    assert!(session.can_redo());
    session.refresh(present(r"E:\fresh;E:\other"));
    assert_eq!(raws(&session), vec![r"E:\fresh", r"E:\other"]);
    assert!(!session.is_dirty());
    assert!(!session.can_undo());
    assert!(!session.can_redo());
}

#[test]
fn an_entry_surviving_refresh_keeps_its_id() {
    let mut session = user_session(r"C:\kept;C:\dropped");
    let kept = session.entries()[0].id();
    let dropped = session.entries()[1].id();
    session.refresh(present(r"C:\inserted;C:\kept"));
    assert_eq!(session.entries()[1].id(), kept);
    assert_ne!(session.entries()[0].id(), kept);
    assert_ne!(session.entries()[0].id(), dropped);
}

// ---- A non-writable Session disables every editing action ----

#[test]
fn a_non_writable_session_rejects_every_editing_action() {
    let mut session = Session::new(Scope::System, present(r"C:\one;C:\two"), false);
    let id = session.entries()[0].id();
    assert!(!session.writable());
    assert!(session.add(r"C:\new").is_none());
    assert!(!session.delete(id));
    assert!(!session.edit(id, r"C:\edited"));
    assert!(!session.move_down(id));
    assert!(!session.move_up(session.entries()[1].id()));
    assert!(!session.set_value_type(ValueType::RegSz));
    assert!(!session.restore(vec![r"D:\x".to_string()], ValueType::RegSz));
    assert!(!session.cancel());
    assert!(session.undo().is_none());
    assert!(session.redo().is_none());
    assert_eq!(raws(&session), vec![r"C:\one", r"C:\two"]);
    assert!(!session.is_dirty());
}
