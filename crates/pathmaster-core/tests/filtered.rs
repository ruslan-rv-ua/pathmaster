//! The Filtered View's matching engine at the crate boundary (v0.2.0 spec §2,
//! §3; ticket impl 03).
//!
//! The rule under test is deliberately small: case-insensitive substring with
//! Unicode case folding, slash-folded (`/`→`\`), and nothing else. Quote
//! stripping, trailing-`\` trimming and `%VAR%` expansion change *what text
//! exists* and stay out — a search for `"` must find the `Quoted` Entries.

use pathmaster_core::diagnostics::Issue;
use pathmaster_core::filtered::{matches, visible_indices, Criteria, Filter};

// ----------------------------------------------------------------- the fold

#[test]
fn a_substring_matches_wherever_it_stands() {
    assert!(matches(r"C:\Program Files\Git\cmd", "git"));
    assert!(matches(r"C:\Program Files\Git\cmd", r"C:\Program"));
    assert!(matches(r"C:\Program Files\Git\cmd", "cmd"));
    assert!(!matches(r"C:\Program Files\Git\cmd", "python"));
}

#[test]
fn case_is_folded_on_both_sides() {
    assert!(matches(r"C:\PROGRAM FILES\GIT", "git"));
    assert!(matches(r"c:\program files\git", "GIT"));
}

#[test]
fn the_fold_is_unicode_not_ascii() {
    // An ASCII fold would be silently case-sensitive for every Cyrillic path
    // (v0.2.0 spec §3).
    assert!(matches(r"C:\Users\Руслан\bin", "руслан"));
    assert!(matches(r"C:\Users\руслан\bin", "РУСЛАН"));
}

#[test]
fn slashes_are_folded_into_backslashes_on_both_sides() {
    assert!(matches(r"C:\Program Files\Git", "files/git"));
    assert!(matches(r"C:/Program Files/Git", r"files\git"));
}

#[test]
fn a_quote_is_searchable_text() {
    // Quote stripping stays out of the fold: a search for `"` finds the
    // Quoted Entries.
    assert!(matches(r#""C:\Program Files\Git""#, "\""));
    assert!(!matches(r"C:\Program Files\Git", "\""));
}

#[test]
fn the_query_is_never_trimmed() {
    // Whitespace is Entry content.
    assert!(matches(r"C:\Program Files\Git", "program f"));
    assert!(!matches(r"C:\ProgramFiles", "program f"));
    assert!(!matches(r"C:\Git", " git"));
}

#[test]
fn an_empty_query_matches_everything() {
    assert!(matches(r"C:\anything", ""));
    assert!(matches("", ""));
}

#[test]
fn an_empty_rendering_matches_only_the_empty_query() {
    assert!(!matches("", "git"));
}

// ----------------------------------------------------------- the visible set

/// Renderings nothing has diagnosed — the Search axis alone, which is what
/// these three are about.
fn undiagnosed<'a>(renderings: &'a [&'a str]) -> Vec<(&'a str, &'a [Issue])> {
    renderings.iter().map(|text| (*text, &[][..])).collect()
}

#[test]
fn visible_indices_are_the_matching_positions_in_order() {
    let renderings = [r"C:\Git\cmd", r"C:\Python", r"C:\git\bin"];
    assert_eq!(
        visible_indices(undiagnosed(&renderings), &Criteria::new("git", Filter::All)),
        vec![0, 2]
    );
}

#[test]
fn an_empty_query_shows_every_entry() {
    let renderings = [r"C:\a", r"C:\b"];
    assert_eq!(
        visible_indices(undiagnosed(&renderings), &Criteria::default()),
        vec![0, 1]
    );
}

#[test]
fn no_match_is_an_empty_set_not_an_error() {
    let renderings = [r"C:\a", r"C:\b"];
    assert_eq!(
        visible_indices(undiagnosed(&renderings), &Criteria::new("zz", Filter::All)),
        Vec::<usize>::new()
    );
}

// ------------------------------------------------------------- the Filter axis

#[test]
fn the_seven_states_are_all_with_issues_and_the_five_entry_level_types() {
    // The submenu's order, and the whole of it: Over-length is Scope-level,
    // flags no Entry, and no state selects it (v0.2.0 spec §4).
    assert_eq!(
        Filter::ALL,
        [
            Filter::All,
            Filter::WithIssues,
            Filter::Missing,
            Filter::Relative,
            Filter::Quoted,
            Filter::Duplicate,
            Filter::Empty,
        ]
    );
}

#[test]
fn the_five_type_states_stand_for_the_issue_types_in_severity_order() {
    // One order for the five words, and it is the rulebook's — the Status
    // column joins them in it and the submenu lists them in it (spec §7).
    assert_eq!(
        Filter::ALL[2..]
            .iter()
            .map(|filter| filter.issue().expect("a type state names its Issue"))
            .collect::<Vec<Issue>>(),
        Issue::SEVERITY
    );
    assert_eq!(Filter::All.issue(), None);
    assert_eq!(Filter::WithIssues.issue(), None);
}

#[test]
fn all_admits_every_entry_healthy_or_not() {
    assert!(Filter::All.admits(&[]));
    assert!(Filter::All.admits(&[Issue::Missing]));
}

#[test]
fn with_issues_means_a_non_empty_status() {
    assert!(!Filter::WithIssues.admits(&[]));
    assert!(Filter::WithIssues.admits(&[Issue::Quoted]));
    assert!(Filter::WithIssues.admits(&[Issue::Missing, Issue::Duplicate]));
}

#[test]
fn a_type_state_admits_an_entry_whose_issue_set_contains_that_type() {
    // "An Entry is visible when its Issue set contains the chosen type" — the
    // set, not its first member: a Missing, Duplicate Entry is both.
    let both = [Issue::Missing, Issue::Duplicate];
    assert!(Filter::Missing.admits(&both));
    assert!(Filter::Duplicate.admits(&both));
    assert!(!Filter::Quoted.admits(&both));
    assert!(!Filter::Missing.admits(&[]));
}

#[test]
fn only_all_leaves_the_view_unnarrowed() {
    assert!(!Filter::All.narrows());
    for filter in Filter::ALL.into_iter().filter(|f| *f != Filter::All) {
        assert!(filter.narrows(), "{filter:?} is a narrowing state");
    }
}

#[test]
fn the_coarse_toggle_goes_all_to_with_issues_and_anything_else_back_to_all() {
    // Ctrl+I's whole contract (v0.2.0 spec §4): the five per-type states are
    // menu-only, and the toggle's way out of any of them is All.
    assert_eq!(Filter::All.toggled(), Filter::WithIssues);
    assert_eq!(Filter::WithIssues.toggled(), Filter::All);
    for filter in Filter::ALL.into_iter().filter(|f| f.narrows()) {
        assert_eq!(filter.toggled(), Filter::All, "{filter:?} toggles out");
    }
}

#[test]
fn every_state_has_a_name_and_the_five_types_reuse_the_status_words() {
    // No new msgids for names (v0.2.0 spec §4): the five type states are the
    // Status column's own words, so the menu, the StatusBar and the column
    // cannot come to call one Issue two things.
    for filter in Filter::ALL {
        assert!(!filter.catalogue_msgid().is_empty());
    }
    for (filter, issue) in Filter::ALL[2..].iter().zip(Issue::SEVERITY) {
        assert_eq!(filter.catalogue_msgid(), issue.catalogue_msgid());
    }
}

#[test]
fn a_state_name_carries_no_mnemonic_because_three_surfaces_share_it() {
    // The names reach the Status column and StatusBar field 0 as well as the
    // submenu, where an `&` would be shown rather than swallowed (spec §4).
    for filter in Filter::ALL {
        assert!(
            !filter.catalogue_msgid().contains('&'),
            "{filter:?} names itself with a mnemonic"
        );
    }
}

// ------------------------------------------------ the two axes, composed by AND

#[test]
fn no_query_and_the_all_state_is_no_filtered_view() {
    // Announcement 1's condition, in one place: empty query AND Filter at All
    // (v0.2.0 spec §13 item 1, the two-part condition completed).
    assert!(!Criteria::default().narrowing());
    assert!(Criteria::new("win", Filter::All).narrowing());
    assert!(Criteria::new("", Filter::WithIssues).narrowing());
    assert!(Criteria::new("win", Filter::Missing).narrowing());
}

#[test]
fn searching_is_the_query_half_alone() {
    // ESC answers to this half: it clears the text, and a view a Filter is
    // still narrowing is still narrowed (v0.2.0 spec §3).
    assert!(!Criteria::new("", Filter::Missing).searching());
    assert!(Criteria::new("win", Filter::All).searching());
}

#[test]
fn an_entry_is_visible_only_when_both_axes_admit_it() {
    let criteria = Criteria::new("git", Filter::Missing);
    assert!(criteria.admits(r"C:\Git\cmd", &[Issue::Missing]));
    // Matches the query, wrong Issue.
    assert!(!criteria.admits(r"C:\Git\cmd", &[Issue::Quoted]));
    // Right Issue, does not match the query.
    assert!(!criteria.admits(r"C:\Python", &[Issue::Missing]));
}

#[test]
fn the_visible_set_is_the_positions_both_axes_leave_standing() {
    let entries = [
        (r"C:\Git\cmd", &[Issue::Missing][..]),
        (r"C:\Python", &[Issue::Missing][..]),
        (r"C:\git\bin", &[][..]),
        (r"C:\git\usr", &[Issue::Duplicate, Issue::Missing][..]),
    ];
    assert_eq!(
        visible_indices(entries, &Criteria::new("git", Filter::Missing)),
        vec![0, 3]
    );
    assert_eq!(
        visible_indices(entries, &Criteria::new("git", Filter::All)),
        vec![0, 2, 3]
    );
    assert_eq!(
        visible_indices(entries, &Criteria::new("", Filter::WithIssues)),
        vec![0, 1, 3]
    );
    assert_eq!(
        visible_indices(entries, &Criteria::default()),
        vec![0, 1, 2, 3]
    );
}

#[test]
fn an_entry_no_pass_has_looked_at_yet_carries_no_issues_and_no_type_admits_it() {
    // The Issue set is the last completed pass's, and before one lands every
    // set is empty — so a type state shows nothing rather than everything
    // (spec §7, FR-diag-async).
    let entries = [(r"C:\Git", &[][..]), (r"C:\Python", &[][..])];
    assert_eq!(
        visible_indices(entries, &Criteria::new("", Filter::WithIssues)),
        Vec::<usize>::new()
    );
    assert_eq!(
        visible_indices(entries, &Criteria::new("", Filter::All)),
        vec![0, 1]
    );
}
