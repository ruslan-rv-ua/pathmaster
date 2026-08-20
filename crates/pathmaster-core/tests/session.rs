//! The Editing Session at the crate boundary (spec §5, ADR-0001, ticket impl-02).

use std::collections::BTreeSet;

use pathmaster_core::msgids;
use pathmaster_core::session::{Operation, Scope, ScopeValue, Session, ValueType};

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

/// Undoes one step and answers with the operation the Checkpoint named.
fn undone(session: &mut Session) -> Operation {
    session.undo().expect("one step to undo").operation
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
    let id = session
        .add(r"C:\new")
        .expect("writable session accepts Add");
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
    assert!(session.batch(Operation::ChangeValueType, |s| {
        s.edit(id, r"C:\%VAR%");
        s.set_value_type(ValueType::RegSz);
        Some(id)
    }));

    assert_eq!(session.undo().expect("one step to undo").focus, Some(id));
    assert_eq!(raws(&session), vec![r"C:\one", r"C:\two"]);
    assert_eq!(session.value_type(), ValueType::RegExpandSz);
    assert!(
        !session.can_undo(),
        "the whole batch was a single Checkpoint"
    );

    assert!(session.redo().is_some());
    assert_eq!(raws(&session), vec![r"C:\%VAR%", r"C:\two"]);
    assert_eq!(session.value_type(), ValueType::RegSz);
}

#[test]
fn a_batch_that_changes_nothing_is_not_an_operation() {
    let mut session = user_session(r"C:\one");
    let id = session.entries()[0].id();
    assert!(!session.batch(Operation::Edit, |s| {
        s.edit(id, r"C:\other");
        s.edit(id, r"C:\one");
        Some(id)
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
    assert_eq!(
        session.undo().expect("one step to undo").focus,
        Some(survivor)
    );
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
    assert!(
        session.is_dirty(),
        "the Baseline stayed at the applied value"
    );
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
    assert!(session.restore(vec![r"D:\restored".to_string()], ValueType::RegSz,));
    assert_eq!(raws(&session), vec![r"D:\restored"]);
    assert_eq!(session.value_type(), ValueType::RegSz);
    assert!(
        session.is_dirty(),
        "Restore edits the Working Copy, never the registry"
    );
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
    session.refresh(present(r"E:\fresh;E:\other"), None);
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
    session.refresh(present(r"C:\inserted;C:\kept"), None);
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

// ---- Each Checkpoint names the operation it undoes (spec §10.1 item 4) ----

#[test]
fn every_operation_names_itself_for_the_announcement() {
    let mut session = user_session(r"C:\one;C:\two");
    let first = session.entries()[0].id();
    let second = session.entries()[1].id();

    session.add(r"C:\three");
    assert_eq!(undone(&mut session), Operation::Add);
    session.edit(first, r"C:\edited");
    assert_eq!(undone(&mut session), Operation::Edit);
    session.delete(second);
    assert_eq!(undone(&mut session), Operation::Delete);
    session.move_down(first);
    assert_eq!(undone(&mut session), Operation::Move);
    session.move_up(second);
    assert_eq!(undone(&mut session), Operation::Move);
    session.set_value_type(ValueType::RegSz);
    assert_eq!(undone(&mut session), Operation::ChangeValueType);
    session.restore(vec![r"D:\restored".to_string()], ValueType::RegSz);
    assert_eq!(undone(&mut session), Operation::Restore);
    session.edit(first, r"C:\dirty");
    session.cancel();
    assert_eq!(undone(&mut session), Operation::Cancel);
}

#[test]
fn redo_announces_the_operation_undo_took_back() {
    let mut session = user_session(r"C:\one");
    session.add(r"C:\two");
    session.undo();
    assert_eq!(
        session.redo().expect("one step to redo").operation,
        Operation::Add,
    );
}

#[test]
fn a_batch_announces_the_one_operation_it_was_named_with() {
    // The convert-or-keep dialog commits an Entry and a Value Type together;
    // the type change is the half the user cannot see, so it is the half the
    // announcement names (spec §6).
    let mut session = user_session(r"C:\one");
    let id = session.entries()[0].id();
    session.batch(Operation::ChangeValueType, |s| {
        s.edit(id, r"C:\%VAR%");
        s.set_value_type(ValueType::RegSz);
        Some(id)
    });
    assert_eq!(undone(&mut session), Operation::ChangeValueType);
}

#[test]
fn a_batch_takes_its_focus_hint_from_the_work_it_ran() {
    // An Add's new Entry does not exist until the batch has run, so the hint
    // cannot be predicted before it — undoing must still land on its
    // neighbour rather than nowhere.
    let mut session = user_session(r"C:\one");
    let survivor = session.entries()[0].id();
    assert!(session.batch(Operation::ChangeValueType, |s| {
        s.set_value_type(ValueType::RegSz);
        s.add(r"C:\%VAR%\bin")
    }));
    assert_eq!(
        session.undo().expect("one step to undo").focus,
        Some(survivor),
    );
}

#[test]
fn an_operation_name_is_a_catalogue_string() {
    // Announcement 4 fills `{operation}` with translated text, so every
    // operation must name a msgid — and the names must not collide.
    let msgids: BTreeSet<&str> = [
        Operation::Add,
        Operation::Edit,
        Operation::Delete,
        Operation::Move,
        Operation::Cancel,
        Operation::ChangeValueType,
        Operation::Restore,
    ]
    .iter()
    .map(|operation| operation.catalogue_msgid())
    .collect();
    assert_eq!(msgids.len(), 7, "each operation names a distinct msgid");
    let registered: BTreeSet<&str> = msgids::REGISTRY.iter().map(|entry| entry.msgid).collect();
    for msgid in msgids {
        assert!(registered.contains(msgid), "{msgid:?} is in the Catalogue");
    }
}

// ---- Undo across the Apply barrier (spec §10.1 item 5) ----

#[test]
fn an_undo_that_re_dirties_a_clean_session_crossed_the_apply_barrier() {
    let mut session = user_session(r"C:\one");
    let id = session.entries()[0].id();
    session.edit(id, r"C:\applied");
    session.mark_applied();
    assert!(!session.is_dirty());
    assert!(
        session.undo().expect("one step to undo").crossed_apply,
        "undoing past an Apply re-dirties the Session — the suffix rides it",
    );
}

#[test]
fn an_undo_that_re_dirties_a_session_no_apply_has_touched_crosses_nothing() {
    // Dirty is a comparison, so an Add and its Delete leave a *clean* Session
    // with two Checkpoints standing behind it. Undoing one re-dirties the
    // Session — but there was never an Apply, so there is no barrier to have
    // crossed, and "unsaved changes" would be news of nothing.
    let mut session = user_session(r"C:\one");
    let added = session
        .add(r"C:\two")
        .expect("writable session accepts Add");
    session.delete(added);
    assert!(!session.is_dirty(), "the pair cancelled out");
    let outcome = session.undo().expect("one step to undo");
    assert!(session.is_dirty(), "the undo brought the Entry back");
    assert!(!outcome.crossed_apply);
}

#[test]
fn an_undo_inside_a_dirty_session_crosses_nothing() {
    let mut session = user_session(r"C:\one");
    let id = session.entries()[0].id();
    session.edit(id, r"C:\once");
    session.edit(id, r"C:\twice");
    assert!(!session.undo().expect("one step to undo").crossed_apply);
}

#[test]
fn an_undo_back_onto_the_baseline_crosses_nothing() {
    let mut session = user_session(r"C:\one");
    let id = session.entries()[0].id();
    session.edit(id, r"C:\changed");
    let outcome = session.undo().expect("one step to undo");
    assert!(!outcome.crossed_apply);
    assert!(!session.is_dirty());
}

#[test]
fn a_redo_can_cross_the_barrier_too() {
    // Redo re-dirties by the same route: Apply, undo, redo lands the Working
    // Copy back off its Baseline.
    let mut session = user_session(r"C:\one");
    let id = session.entries()[0].id();
    session.edit(id, r"C:\edited");
    session.undo();
    session.mark_applied();
    assert!(!session.is_dirty());
    assert!(session.redo().expect("one step to redo").crossed_apply);
}

// ---- Where focus lands after a Refresh (FR-refresh) ----

#[test]
fn refresh_keeps_focus_on_the_entry_that_survived_it() {
    let mut session = user_session(r"C:\one;C:\two");
    let focused = session.entries()[1].id();
    let landing = session.refresh(present(r"C:\two;C:\three"), Some(focused));
    assert_eq!(landing, Some(focused));
    assert_eq!(session.entries()[0].id(), focused);
}

#[test]
fn refresh_falls_back_to_the_nearest_neighbour_by_index() {
    let mut session = user_session(r"C:\one;C:\gone;C:\three");
    let focused = session.entries()[1].id();
    let landing = session.refresh(present(r"E:\a;E:\b;E:\c"), Some(focused));
    assert_eq!(
        landing,
        Some(session.entries()[1].id()),
        "the Entry now standing where the focused one stood",
    );
}

#[test]
fn refresh_clamps_the_neighbour_to_the_new_last_row() {
    let mut session = user_session(r"C:\one;C:\two;C:\three");
    let focused = session.entries()[2].id();
    let landing = session.refresh(present(r"E:\only"), Some(focused));
    assert_eq!(landing, Some(session.entries()[0].id()));
}

#[test]
fn refresh_of_an_emptied_scope_hands_focus_back_to_the_list() {
    let mut session = user_session(r"C:\one");
    let focused = session.entries()[0].id();
    assert_eq!(session.refresh(ScopeValue::Absent, Some(focused)), None);
}

#[test]
fn refresh_with_nothing_focused_lands_nowhere() {
    let mut session = user_session(r"C:\one");
    assert_eq!(session.refresh(present(r"C:\one"), None), None);
}
