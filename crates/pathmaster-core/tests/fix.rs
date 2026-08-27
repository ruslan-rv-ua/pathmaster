//! Fix Issues' rulebook at the crate boundary (v0.2.0 spec §7, ticket impl-09).
//!
//! Three questions, and this file is the whole of all three: which Entries the
//! repair surface offers a row for and what it proposes to do to each, which of
//! those rows start checked, and what carrying the chosen ones out does to a
//! Session. Everything else about the dialog — its columns, its Space toggle,
//! where focus lands afterwards — is the window's, and the words every cell
//! shows are the Catalogue's.
//!
//! The machine is injected, as it is for the diagnostic rulebook: the one
//! filesystem fact the defaults ask for is "is this root a fixed disk?", and a
//! rule whose answer depended on the drives plugged into the test runner would
//! not be a rule.

use pathmaster_core::diagnostics::Issue;
use pathmaster_core::fix::{fixable, repair, Action, DriveTypes, Plan, Row};
use pathmaster_core::session::{Entry, EntryId, Operation, Scope, ScopeValue, Session, ValueType};

/// The drives this run pretends to have: every root named here is fixed, and
/// every other one is not.
struct Drives(&'static [&'static str]);

impl DriveTypes for Drives {
    fn is_fixed_root(&self, path: &str) -> bool {
        self.0
            .iter()
            .any(|root| path.to_uppercase().starts_with(&root.to_uppercase()))
    }
}

/// The machine the shape tests run on: `C:` is the fixed disk, `E:` is not.
const DRIVES: Drives = Drives(&[r"C:\"]);

/// A Session over `raws`, which is the only way to come by Entry ids — a row
/// carries one and nothing else identifies the Entry it will repair.
fn session(raws: &[&str]) -> Session {
    Session::new(
        Scope::User,
        ScopeValue::Present {
            value_type: ValueType::RegExpandSz,
            raw: raws.join(";"),
        },
        true,
    )
}

/// The plan over every Entry of `session`, with `issues` on the Entry at each
/// listed position — the shape the last completed pass hands over.
fn plan(session: &Session, issues: &[(usize, &[Issue])]) -> Plan {
    Plan::of(
        session.entries().iter().enumerate().map(|(index, entry)| {
            let flagged = issues
                .iter()
                .find(|(at, _)| *at == index)
                .map_or(&[][..], |(_, flagged)| *flagged);
            (entry.id(), entry.raw(), flagged)
        }),
        &DRIVES,
    )
}

/// The id of the Entry at `index`, for the tests that check a row leads back
/// to the right one.
fn id_at(session: &Session, index: usize) -> EntryId {
    session.entries()[index].id()
}

/// A Session's Entries as text, in order.
fn raws(session: &Session) -> Vec<&str> {
    session.entries().iter().map(Entry::raw).collect()
}

// ------------------------------------------------------- which Entries get a row

#[test]
fn the_four_fixable_types_each_earn_a_row() {
    for issue in [
        Issue::Missing,
        Issue::Duplicate,
        Issue::Empty,
        Issue::Quoted,
    ] {
        assert!(
            Action::proposed(&[issue]).is_some(),
            "{issue:?} proposes no repair"
        );
    }
}

#[test]
fn a_healthy_entry_has_no_row() {
    assert_eq!(Action::proposed(&[]), None);
}

#[test]
fn a_relative_only_entry_has_no_row() {
    // Qualification needs a base directory only the user knows, so there is no
    // repair a tool may guess at — and a row that can fix nothing is noise.
    assert_eq!(Action::proposed(&[Issue::Relative]), None);
}

#[test]
fn relative_still_rides_along_with_a_fixable_type() {
    // The exclusion is of the Entry flagged *only* Relative, never of the
    // finding: a Relative, Quoted Entry has quotes to remove like any other.
    assert_eq!(
        Action::proposed(&[Issue::Relative, Issue::Quoted]),
        Some(Action::RemoveQuotes)
    );
}

#[test]
fn the_plan_holds_one_row_per_fixable_entry_and_no_other() {
    let session = session(&[r"C:\Windows", r"C:\nope", "bin", r"C:\dup"]);
    let plan = plan(
        &session,
        &[
            (1, &[Issue::Missing]),
            (2, &[Issue::Relative]),
            (3, &[Issue::Duplicate]),
        ],
    );

    let ids: Vec<EntryId> = plan.rows().iter().map(|row| row.id).collect();
    assert_eq!(ids, vec![id_at(&session, 1), id_at(&session, 3)]);
}

// --------------------------------------------------------- the computed action

#[test]
fn each_of_the_three_deletions_proposes_delete_entry() {
    for issue in [Issue::Missing, Issue::Duplicate, Issue::Empty] {
        assert_eq!(Action::proposed(&[issue]), Some(Action::Delete));
    }
}

#[test]
fn quoted_alone_proposes_removing_the_quotes() {
    assert_eq!(
        Action::proposed(&[Issue::Quoted]),
        Some(Action::RemoveQuotes)
    );
}

#[test]
fn a_deletion_beats_a_quote_repair_on_one_entry() {
    // One row per Entry, one computed action: deletion cures Quoted too, so
    // there is never a row that both deletes and repairs.
    assert_eq!(
        Action::proposed(&[Issue::Missing, Issue::Quoted]),
        Some(Action::Delete)
    );
    assert_eq!(
        Action::proposed(&[Issue::Quoted, Issue::Duplicate]),
        Some(Action::Delete)
    );
}

#[test]
fn removing_quotes_takes_every_quote_in_the_entry() {
    // Every `"`, not one surrounding pair: `"` is illegal in a Windows file
    // name, so no quote anywhere in the text can be path content.
    assert_eq!(
        Action::RemoveQuotes.leaves(r#""C:\Program Files"\bin""#),
        Some(r"C:\Program Files\bin".to_string())
    );
}

#[test]
fn a_deletion_leaves_no_text_behind() {
    assert_eq!(Action::Delete.leaves(r"C:\nope"), None);
}

// ------------------------------------------------------------ what a row shows

#[test]
fn a_row_carries_the_entrys_original_position() {
    // §2.1's convention, the same `#` the main list shows: the Working Copy's
    // 1-based position, never the row's own place in this dialog.
    let session = session(&[r"C:\Windows", r"C:\nope", r"C:\also-nope"]);
    let plan = plan(&session, &[(1, &[Issue::Missing]), (2, &[Issue::Missing])]);

    let positions: Vec<usize> = plan.rows().iter().map(|row| row.position).collect();
    assert_eq!(positions, vec![2, 3]);
}

#[test]
fn the_path_a_row_shows_is_always_the_raw_text() {
    // Whatever the Expansion Mode: the dialog shows what will be deleted or
    // repaired, and the %VAR% default rule must be visible in the row it
    // judges. The plan cannot be handed a mode, which is how that is kept.
    let session = session(&[r#""%JAVA_HOME%\bin""#]);
    let plan = plan(&session, &[(0, &[Issue::Missing, Issue::Quoted])]);

    assert_eq!(plan.rows()[0].raw, r#""%JAVA_HOME%\bin""#);
}

#[test]
fn a_row_carries_the_whole_flagged_set_the_issue_column_joins() {
    let session = session(&[r#""C:\nope""#]);
    let plan = plan(&session, &[(0, &[Issue::Missing, Issue::Quoted])]);

    assert_eq!(plan.rows()[0].issues, vec![Issue::Missing, Issue::Quoted]);
}

// --------------------------------------------- the defaults: the Disk Cleanup principle

#[test]
fn removing_quotes_starts_checked() {
    let session = session(&[r#""E:\removable""#]);
    let plan = plan(&session, &[(0, &[Issue::Quoted])]);

    // Guaranteed behaviour-preserving, whatever the drive it names.
    assert!(plan.rows()[0].checked);
}

#[test]
fn a_duplicate_or_empty_deletion_starts_checked() {
    let session = session(&[r"E:\tools", "   "]);
    let plan = plan(&session, &[(0, &[Issue::Duplicate]), (1, &[Issue::Empty])]);

    assert!(plan.rows().iter().all(|row| row.checked));
}

#[test]
fn a_missing_deletion_starts_checked_on_a_fixed_local_root() {
    let session = session(&[r"C:\gone"]);
    let plan = plan(&session, &[(0, &[Issue::Missing])]);

    assert!(plan.rows()[0].checked);
}

#[test]
fn a_missing_deletion_starts_unchecked_when_the_raw_text_carries_a_variable() {
    // The reference may name a directory this run simply does not define —
    // a machine's own %VAR% is not evidence the Entry is stale.
    let session = session(&[r"%TOOLS%\bin"]);
    let plan = plan(&session, &[(0, &[Issue::Missing])]);

    assert!(!plan.rows()[0].checked);
}

#[test]
fn a_missing_deletion_starts_unchecked_on_a_non_fixed_root() {
    // Removable media that is not in the drive right now is absent, not stale.
    let session = session(&[r"E:\tools"]);
    let plan = plan(&session, &[(0, &[Issue::Missing])]);

    assert!(!plan.rows()[0].checked);
}

#[test]
fn the_root_is_read_past_the_entrys_own_quotes() {
    let session = session(&[r#""C:\gone""#]);
    let plan = plan(&session, &[(0, &[Issue::Missing, Issue::Quoted])]);

    assert!(plan.rows()[0].checked);
}

#[test]
fn a_deletion_the_duplicate_flag_earns_starts_checked_whatever_the_drive_says() {
    // The cautious rule is about *Missing*: a second copy of an Entry is
    // redundant whether or not the path it names is there.
    let session = session(&[r"%TOOLS%\bin", r"E:\tools"]);
    let plan = plan(
        &session,
        &[
            (0, &[Issue::Missing, Issue::Duplicate]),
            (1, &[Issue::Missing, Issue::Duplicate]),
        ],
    );

    assert!(plan.rows().iter().all(|row| row.checked));
}

// ------------------------------------------------------------------- enablement

#[test]
fn fixable_counts_the_rows_the_dialog_would_show() {
    let counted = fixable([
        &[][..],
        &[Issue::Relative],
        &[Issue::Missing],
        &[Issue::Quoted, Issue::Relative],
    ]);
    assert_eq!(counted, 2);
}

#[test]
fn a_scope_whose_issues_are_all_relative_has_nothing_to_fix() {
    // Not merely "Issues exist": an all-Relative Scope would open an empty
    // dialog, which is what the enablement rule exists to prevent.
    assert_eq!(fixable([&[Issue::Relative][..], &[Issue::Relative]]), 0);
}

#[test]
fn an_empty_plan_knows_it_is_empty() {
    let session = session(&[r"C:\Windows"]);
    let plan = plan(&session, &[]);

    assert!(plan.is_empty());
    assert_eq!(plan.rows(), &[] as &[Row]);
}

// ------------------------------------------------- carrying the chosen rows out

/// The chosen rows of `plan` at the listed row indices, in the plan's order —
/// what the dialog hands back after reading its checkboxes.
fn chosen(plan: &Plan, rows: &[usize]) -> Vec<(EntryId, Action)> {
    rows.iter()
        .map(|&row| (plan.rows()[row].id, plan.rows()[row].action))
        .collect()
}

#[test]
fn one_repair_covers_every_chosen_row_and_answers_how_many_changed() {
    let mut session = session(&[r"C:\one", r#""C:\two""#, r"C:\gone", r"C:\four"]);
    let plan = plan(&session, &[(1, &[Issue::Quoted]), (2, &[Issue::Missing])]);
    let chosen = chosen(&plan, &[0, 1]);

    assert_eq!(repair(&mut session, &chosen), 2);
    assert_eq!(
        raws(&session),
        vec![r"C:\one", r"C:\two", r"C:\four"],
        "the quoted Entry was repaired in place and the missing one deleted"
    );
}

#[test]
fn every_chosen_row_lands_in_one_checkpoint() {
    // §7: one user-visible operation, one Checkpoint — which is what makes a
    // single Ctrl+Z restore every Entry the dialog touched.
    let mut session = session(&[r"C:\one", r#""C:\two""#, r"C:\gone"]);
    let plan = plan(&session, &[(1, &[Issue::Quoted]), (2, &[Issue::Missing])]);
    repair(&mut session, &chosen(&plan, &[0, 1]));

    let outcome = session.undo().expect("one step to undo");
    assert_eq!(outcome.operation, Operation::FixIssues);
    assert_eq!(raws(&session), vec![r"C:\one", r#""C:\two""#, r"C:\gone"]);
    assert!(!session.can_undo(), "the whole repair was one Checkpoint");
}

#[test]
fn nothing_chosen_is_not_an_operation() {
    // Zero rows checked closes the dialog like Cancel: no Checkpoint, and the
    // count Announcement 12 would speak is zero, so nothing is spoken.
    let mut session = session(&[r"C:\one"]);
    assert_eq!(repair(&mut session, &[]), 0);
    assert!(!session.can_undo());
    assert!(!session.is_dirty());
}

#[test]
fn rows_resolve_by_identity_and_never_by_position() {
    // Deleting the first chosen row shifts every later Entry up. Only identity
    // survives that, which is why the dialog answers with ids.
    let mut session = session(&[r"C:\gone", r"C:\also-gone", r#""C:\quoted""#]);
    let plan = plan(
        &session,
        &[
            (0, &[Issue::Missing]),
            (1, &[Issue::Missing]),
            (2, &[Issue::Quoted]),
        ],
    );

    assert_eq!(repair(&mut session, &chosen(&plan, &[0, 1, 2])), 3);
    assert_eq!(raws(&session), vec![r"C:\quoted"]);
}

#[test]
fn an_entry_an_earlier_row_took_away_is_passed_over_rather_than_counted() {
    // Unreachable while the dialog is modal, and answered rather than assumed
    // away: the same id chosen twice deletes once.
    let mut session = session(&[r"C:\one", r"C:\gone"]);
    let plan = plan(&session, &[(1, &[Issue::Missing])]);
    let row = &plan.rows()[0];
    let twice = [(row.id, row.action), (row.id, row.action)];

    assert_eq!(repair(&mut session, &twice), 1);
    assert_eq!(raws(&session), vec![r"C:\one"]);
}

#[test]
fn the_checkpoint_hints_the_first_surviving_neighbour() {
    // Delete's law, asked of the whole batch: the Entry left standing where
    // the first repaired row stood is what an undo of "Fixing issues" lands on.
    let mut session = session(&[r"C:\one", r"C:\gone", r"C:\three"]);
    let plan = plan(&session, &[(1, &[Issue::Missing])]);
    let neighbour = session.entries()[2].id();
    repair(&mut session, &chosen(&plan, &[0]));

    assert_eq!(
        session.undo().expect("one step to undo").focus,
        Some(neighbour),
        "the Entry that took the deleted row's place is the hint"
    );
}

#[test]
fn a_repair_that_empties_the_scope_hints_no_entry_at_all() {
    let mut session = session(&[r"C:\gone"]);
    let plan = plan(&session, &[(0, &[Issue::Missing])]);
    repair(&mut session, &chosen(&plan, &[0]));

    assert!(session.entries().is_empty());
    assert_eq!(session.undo().expect("one step to undo").focus, None);
}

#[test]
fn a_session_nobody_may_edit_is_repaired_by_nothing() {
    // The menu item is already closed over a non-writable Session; this is the
    // rule underneath it, answered rather than assumed away.
    let readonly = Session::new(
        Scope::System,
        ScopeValue::Present {
            value_type: ValueType::RegExpandSz,
            raw: r"C:\gone".to_string(),
        },
        false,
    );
    let mut readonly = readonly;
    let id = readonly.entries()[0].id();

    assert_eq!(repair(&mut readonly, &[(id, Action::Delete)]), 0);
    assert_eq!(raws(&readonly), vec![r"C:\gone"]);
    assert!(!readonly.can_undo());
}
