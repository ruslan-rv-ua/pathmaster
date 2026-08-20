//! The diagnostic rulebook at the crate boundary (spec §7, ticket impl-09).
//!
//! The five per-Entry Issue types over the two Working Copies, evaluated in
//! runtime order — System first, then User, each left to right. Issues are a
//! derived view: the pass takes Entry text and gives back findings, and nothing
//! it computes is ever stored in a Working Copy or a Checkpoint.

use std::cell::RefCell;
use std::collections::BTreeSet;

use pathmaster_core::diagnostics::{diagnose, Existence, Filesystem, Findings, Issue, RootKind};
use pathmaster_core::msgids;
use pathmaster_core::normalize::Environment;
use pathmaster_core::session::{Entry, Scope, ScopeValue, Session, ValueType};
use pathmaster_core::thresholds::Overlength;

// ---- The two injected adapters, faked ----

struct Env(&'static [(&'static str, &'static str)]);

impl Environment for Env {
    fn lookup(&self, name: &str) -> Option<String> {
        self.0
            .iter()
            .find(|(key, _)| key.eq_ignore_ascii_case(name))
            .map(|(_, value)| (*value).to_string())
    }
}

const ENV: Env = Env(&[("SystemRoot", r"C:\Windows"), ("JAVA_HOME", r"C:\Java")]);

/// A filesystem that answers from lists and remembers every path it was asked
/// about — "never probed" is a rule with teeth only if a test can see it.
#[derive(Default)]
struct Fs {
    directories: Vec<String>,
    files: Vec<String>,
    denied: Vec<String>,
    network: Vec<String>,
    probed: RefCell<Vec<String>>,
}

impl Fs {
    fn new() -> Fs {
        Fs::default()
    }

    fn directory(mut self, path: &str) -> Fs {
        self.directories.push(path.to_string());
        self
    }

    fn file(mut self, path: &str) -> Fs {
        self.files.push(path.to_string());
        self
    }

    fn denied(mut self, path: &str) -> Fs {
        self.denied.push(path.to_string());
        self
    }

    /// A root the adapter would classify `DRIVE_REMOTE`, or a UNC prefix.
    fn network(mut self, prefix: &str) -> Fs {
        self.network.push(prefix.to_string());
        self
    }

    fn probed(&self) -> Vec<String> {
        self.probed.borrow().clone()
    }
}

impl Filesystem for Fs {
    fn root_kind(&self, path: &str) -> RootKind {
        // The real adapter asks `GetDriveTypeW` and reads the UNC prefix; a
        // fake only has to answer, so the tests say which roots are remote.
        let remote = self
            .network
            .iter()
            .any(|prefix| path.to_lowercase().starts_with(&prefix.to_lowercase()));
        if remote {
            RootKind::Network
        } else {
            RootKind::Local
        }
    }

    fn probe(&self, path: &str) -> Existence {
        self.probed.borrow_mut().push(path.to_string());
        // Win32 answers for `C:\dir\` exactly as it does for `C:\dir`, and is
        // case-insensitive; the fake matches on the same terms.
        let asked = path.trim_end_matches(['\\', '/']);
        let listed = |paths: &[String]| {
            paths
                .iter()
                .any(|p| p.trim_end_matches(['\\', '/']).eq_ignore_ascii_case(asked))
        };
        if listed(&self.directories) {
            Existence::Directory
        } else if listed(&self.files) {
            Existence::File
        } else if listed(&self.denied) {
            Existence::AccessDenied
        } else {
            Existence::NotFound
        }
    }
}

const NONE: &[&str] = &[];

/// The Issues of one User Entry — the shape most rules are stated in.
fn issues(entry: &str, fs: &Fs) -> Vec<Issue> {
    diagnose(NONE, &[entry], &ENV, fs)
        .scope(Scope::User)
        .issues(0)
        .to_vec()
}

/// Every User Entry's Issues, in list order.
fn user_issues(entries: &[&str], fs: &Fs) -> Vec<Vec<Issue>> {
    let diagnosis = diagnose(NONE, entries, &ENV, fs);
    (0..entries.len())
        .map(|i| diagnosis.scope(Scope::User).issues(i).to_vec())
        .collect()
}

fn healthy() -> Vec<Issue> {
    Vec::new()
}

// ---- Empty ----

#[test]
fn a_zero_length_entry_is_empty() {
    // What a `;;` or a trailing `;` decodes to.
    assert_eq!(issues("", &Fs::new()), vec![Issue::Empty]);
}

#[test]
fn a_whitespace_only_entry_is_empty() {
    assert_eq!(issues("   ", &Fs::new()), vec![Issue::Empty]);
    assert_eq!(issues("\t", &Fs::new()), vec![Issue::Empty]);
}

#[test]
fn a_scope_with_no_entries_reports_nothing() {
    // An Absent or empty Scope decodes to zero Entries, not one empty Entry.
    let diagnosis = diagnose(NONE, NONE, &ENV, &Fs::new());
    assert!(diagnosis.scope(Scope::User).is_empty());
    assert_eq!(diagnosis.scope(Scope::User).issue_count(), 0);
}

#[test]
fn an_empty_entry_carries_nothing_else() {
    // Empty is exclusive: two empties are not also duplicates, and no empty is
    // relative, missing or probed.
    let fs = Fs::new();
    assert_eq!(
        user_issues(&["", "   ", ""], &fs),
        vec![vec![Issue::Empty], vec![Issue::Empty], vec![Issue::Empty]],
    );
    assert!(fs.probed().is_empty());
}

// ---- Quoted ----

#[test]
fn an_entry_containing_a_quote_is_quoted() {
    let fs = Fs::new().directory(r"C:\Program Files\foo");
    assert_eq!(
        issues(r#""C:\Program Files\foo""#, &fs),
        vec![Issue::Quoted],
    );
}

#[test]
fn a_quote_anywhere_in_the_entry_counts() {
    let fs = Fs::new().directory(r#"C:\od"d"#);
    assert!(issues(r#"C:\od"d"#, &fs).contains(&Issue::Quoted));
}

#[test]
fn the_existence_check_reads_the_quote_stripped_expanded_path() {
    // The raw Entry still round-trips untouched; only the probe reads past the
    // quotes, and past `%VAR%`.
    let fs = Fs::new().directory(r"C:\Windows\System32");
    assert_eq!(
        issues(r#""%SystemRoot%\System32""#, &fs),
        vec![Issue::Quoted],
    );
    assert_eq!(fs.probed(), vec![r"C:\Windows\System32".to_string()]);
}

#[test]
fn the_existence_check_reads_the_expanded_text_verbatim() {
    // No slash conversion on the way to the filesystem: `/` and `\` name the
    // same directory to Win32, and `\\?\` paths mean it when they say `/`.
    let fs = Fs::new();
    issues("C:/tools", &fs);
    assert_eq!(fs.probed(), vec!["C:/tools".to_string()]);
}

// ---- Relative ----

#[test]
fn an_entry_that_is_not_fully_qualified_is_relative() {
    let fs = Fs::new();
    for entry in [".", "..", "tools", r"\foo", "C:foo", r"..\bin"] {
        assert_eq!(issues(entry, &fs), vec![Issue::Relative], "{entry}");
    }
}

#[test]
fn a_fully_qualified_entry_is_not_relative() {
    let fs = Fs::new()
        .directory(r"C:\tools")
        .directory("C:/tools")
        .directory(r"\\?\C:\tools")
        .directory(r"\\server\share\tools")
        .network(r"\\server");
    for entry in [
        r"C:\tools",
        "C:/tools",
        r"\\?\C:\tools",
        r"\\server\share\tools",
    ] {
        assert_eq!(issues(entry, &fs), healthy(), "{entry}");
    }
}

#[test]
fn an_expanded_reference_is_judged_on_what_it_expands_to() {
    // `%SystemRoot%\System32` is the commonest Entry there is — judging it
    // before expansion would call every one of them relative.
    let fs = Fs::new().directory(r"C:\Windows\System32");
    assert_eq!(issues(r"%SystemRoot%\System32", &fs), healthy());
}

#[test]
fn a_relative_entry_skips_the_existence_check() {
    // Its resolution depends on process state, so Relative and Missing never
    // co-occur — and nothing is probed.
    let fs = Fs::new();
    let issues = issues(r"..\bin", &fs);
    assert_eq!(issues, vec![Issue::Relative]);
    assert!(!issues.contains(&Issue::Missing));
    assert!(fs.probed().is_empty());
}

// ---- Missing ----

#[test]
fn an_existing_directory_is_healthy() {
    assert_eq!(
        issues(r"C:\Windows", &Fs::new().directory(r"C:\Windows")),
        healthy(),
    );
}

#[test]
fn a_path_that_does_not_exist_is_missing() {
    assert_eq!(issues(r"C:\gone", &Fs::new()), vec![Issue::Missing]);
}

#[test]
fn a_path_that_names_a_file_is_missing() {
    // A file Entry is inert: the path search appends `\name.exe` to it.
    assert_eq!(
        issues(
            r"C:\tools\thing.exe",
            &Fs::new().file(r"C:\tools\thing.exe")
        ),
        vec![Issue::Missing],
    );
}

#[test]
fn access_denied_is_not_missing() {
    // The object exists; calling it missing is the .NET `File.Exists` mistake.
    assert_eq!(
        issues(r"C:\locked", &Fs::new().denied(r"C:\locked")),
        healthy(),
    );
}

#[test]
fn a_network_rooted_entry_is_never_probed_and_never_flags() {
    // A dead UNC path blocks 20-60 s uncancellably (research/13), so v0.1.0
    // classifies the root and stops there.
    let fs = Fs::new().network(r"\\nas").network("Z:");
    assert_eq!(issues(r"\\nas\share\tools", &fs), healthy());
    assert_eq!(issues(r"Z:\tools", &fs), healthy());
    assert!(fs.probed().is_empty());
}

#[test]
fn an_undefined_reference_flags_missing_not_relative() {
    // The literal text fails the existence check — no seventh Issue type, and
    // the shape of text carrying an unresolved reference is not judged.
    let fs = Fs::new();
    assert_eq!(issues(r"%NOPE%\bin", &fs), vec![Issue::Missing]);
    assert_eq!(fs.probed(), vec![r"%NOPE%\bin".to_string()]);
}

#[test]
fn an_unresolved_reference_past_the_start_still_leaves_the_shape_legible() {
    // Only a reference in the leading position makes "is this fully qualified?"
    // unanswerable. `tools\%NOPE%` is a bare name whatever `%NOPE%` was, so it
    // is Relative like any other — and skips the existence check with them.
    let fs = Fs::new();
    for entry in [r"tools\%NOPE%", r"..\%NOPE%"] {
        assert_eq!(issues(entry, &fs), vec![Issue::Relative], "{entry}");
    }
    assert!(fs.probed().is_empty());
}

#[test]
fn a_directory_literally_named_like_a_reference_still_exists() {
    // Unresolved is not the same as absent: `%NOPE%` is a legal directory name.
    let fs = Fs::new().directory(r"C:\%NOPE%\bin");
    assert_eq!(issues(r"C:\%NOPE%\bin", &fs), healthy());
}

// ---- Duplicate ----

#[test]
fn the_first_occurrence_is_canonical_and_every_later_copy_flags() {
    let fs = Fs::new().directory(r"C:\tools");
    assert_eq!(
        user_issues(&[r"C:\tools", r"C:\tools", r"C:\tools"], &fs),
        vec![healthy(), vec![Issue::Duplicate], vec![Issue::Duplicate]],
    );
}

#[test]
fn duplicates_are_equal_normalisations_not_equal_text() {
    let fs = Fs::new().directory(r"C:\Windows\System32");
    let entries = [
        r"C:\Windows\System32",
        r"c:\windows\system32\",
        "C:/Windows/System32",
        r#""C:\Windows\System32""#,
        r"%SystemRoot%\System32",
    ];
    let issues = user_issues(&entries, &fs);
    assert_eq!(issues[0], healthy());
    for (index, entry) in entries.iter().enumerate().skip(1) {
        assert!(issues[index].contains(&Issue::Duplicate), "{entry}");
    }
}

#[test]
fn different_paths_are_not_duplicates() {
    let fs = Fs::new().directory(r"C:\one").directory(r"C:\two");
    assert_eq!(
        user_issues(&[r"C:\one", r"C:\two"], &fs),
        vec![healthy(), healthy()],
    );
}

#[test]
fn evaluation_order_is_runtime_order_so_the_user_copy_carries_a_cross_scope_duplicate() {
    // Windows merges System first, then User: the System copy is the one that
    // wins at runtime, so it is the clean one.
    let fs = Fs::new().directory(r"C:\shared");
    let diagnosis = diagnose(&[r"C:\shared"], &[r"C:\shared"], &ENV, &fs);
    assert_eq!(diagnosis.scope(Scope::System).issues(0), &[] as &[Issue]);
    assert_eq!(diagnosis.scope(Scope::User).issues(0), &[Issue::Duplicate]);
}

#[test]
fn a_cross_scope_duplicate_is_found_through_normalisation_too() {
    let fs = Fs::new().directory(r"C:\Java\bin");
    let diagnosis = diagnose(&[r"C:\Java\bin"], &[r"%JAVA_HOME%\bin\"], &ENV, &fs);
    assert_eq!(diagnosis.scope(Scope::System).issues(0), &[] as &[Issue]);
    assert_eq!(diagnosis.scope(Scope::User).issues(0), &[Issue::Duplicate]);
}

#[test]
fn a_duplicate_of_a_relative_entry_is_still_a_duplicate() {
    let fs = Fs::new();
    assert_eq!(
        user_issues(&[".", "."], &fs),
        vec![
            vec![Issue::Relative],
            vec![Issue::Relative, Issue::Duplicate],
        ],
    );
}

// ---- Coexistence and severity order ----

#[test]
fn issues_are_listed_most_severe_first() {
    // Missing > Relative > Quoted > Duplicate > Empty.
    let fs = Fs::new();
    let issues = user_issues(&[r#""C:\gone""#, r#""C:\gone""#], &fs);
    assert_eq!(
        issues[1],
        vec![Issue::Missing, Issue::Quoted, Issue::Duplicate]
    );
}

#[test]
fn quoted_co_occurs_freely() {
    let fs = Fs::new();
    assert_eq!(
        issues(r#""C:\gone""#, &fs),
        vec![Issue::Missing, Issue::Quoted]
    );
    assert_eq!(
        issues(r#""tools""#, &fs),
        vec![Issue::Relative, Issue::Quoted]
    );
}

#[test]
fn relative_and_missing_never_co_occur() {
    let fs = Fs::new();
    for entry in [".", "..", "tools", r"\foo", "C:foo"] {
        let issues = issues(entry, &fs);
        assert!(issues.contains(&Issue::Relative), "{entry}");
        assert!(!issues.contains(&Issue::Missing), "{entry}");
    }
}

#[test]
fn the_severity_order_is_the_declared_one() {
    assert_eq!(
        Issue::SEVERITY,
        [
            Issue::Missing,
            Issue::Relative,
            Issue::Quoted,
            Issue::Duplicate,
            Issue::Empty,
        ],
    );
}

// ---- The derived view itself ----

#[test]
fn the_view_is_one_list_per_entry_in_list_order() {
    let fs = Fs::new().directory(r"C:\tools");
    let diagnosis = diagnose(NONE, &[r"C:\tools", "", r"C:\gone"], &ENV, &fs);
    assert_eq!(diagnosis.scope(Scope::User).len(), 3);
    assert_eq!(diagnosis.scope(Scope::User).issues(0), &[] as &[Issue]);
    assert_eq!(diagnosis.scope(Scope::User).issues(1), &[Issue::Empty]);
    assert_eq!(diagnosis.scope(Scope::User).issues(2), &[Issue::Missing]);
}

#[test]
fn an_index_past_the_end_has_no_issues() {
    // A pass can only ever be shorter than a Working Copy that has moved on;
    // the view answers rather than panics.
    let diagnosis = diagnose(NONE, &[r"C:\gone"], &ENV, &Fs::new());
    assert_eq!(diagnosis.scope(Scope::User).issues(7), &[] as &[Issue]);
}

#[test]
fn the_issue_count_is_every_finding_not_every_flagged_entry() {
    let fs = Fs::new();
    let diagnosis = diagnose(NONE, &[r#""C:\gone""#, r"C:\also-gone"], &ENV, &fs);
    assert_eq!(diagnosis.scope(Scope::User).issue_count(), 3);
}

#[test]
fn each_scope_is_addressable_by_name() {
    let fs = Fs::new();
    let diagnosis = diagnose(&[""], &[r"C:\gone", ""], &ENV, &fs);
    assert_eq!(diagnosis.scope(Scope::System).len(), 1);
    assert_eq!(diagnosis.scope(Scope::User).len(), 2);
    assert_eq!(diagnosis.scope(Scope::System).issue_count(), 1);
    assert_eq!(diagnosis.scope(Scope::User).issue_count(), 2);
}

// ---- The merged length rides along with the pass ----

#[test]
fn the_merged_length_is_measured_over_the_expanded_working_copies() {
    // `C:\Windows\System32` (19) + `;` + `C:\Java\bin` (11).
    let diagnosis = diagnose(
        &[r"%SystemRoot%\System32"],
        &[r"%JAVA_HOME%\bin"],
        &ENV,
        &Fs::new(),
    );
    assert_eq!(diagnosis.merged_length(), 31);
    assert_eq!(diagnosis.overlength(), Overlength::Within);
}

#[test]
fn the_merged_length_joins_a_scopes_entries_with_semicolons() {
    // `a;bb` (4) + `;` + `ccc` (3).
    let diagnosis = diagnose(&["a", "bb"], &["ccc"], &ENV, &Fs::new());
    assert_eq!(diagnosis.merged_length(), 8);
}

#[test]
fn a_merged_length_past_the_cmd_limit_is_classified_but_never_flags_an_entry() {
    let long = format!(r"C:\{}", "x".repeat(9_000));
    let fs = Fs::new().directory(&long);
    let diagnosis = diagnose(NONE, &[long.as_str()], &ENV, &fs);
    assert!(diagnosis.merged_length() > 8_191);
    assert_eq!(diagnosis.overlength(), Overlength::CmdLimit);
    assert_eq!(diagnosis.scope(Scope::User).issues(0), &[] as &[Issue]);
    assert_eq!(diagnosis.scope(Scope::User).issue_count(), 0);
}

// ---- Findings: a pass held against a Working Copy that has moved on ----
//
// A pass is asynchronous (spec §7, FR-diag-async), so between an edit and the
// next pass landing the screen shows Entries the last pass did not run over.
// `Findings` is what the Status column reads in that window: it keys the pass
// by Entry id, not by row, so an Entry that only moved keeps its findings and
// an Entry whose text changed has none until the new pass lands.

/// A Session over one Scope's raw value, the shape a Working Copy arrives in.
fn session(raw: &str) -> Session {
    Session::new(
        Scope::User,
        ScopeValue::Present {
            value_type: ValueType::RegExpandSz,
            raw: raw.to_string(),
        },
        true,
    )
}

/// The findings of one pass over `session`'s Working Copy.
fn findings_over(session: &Session, fs: &Fs) -> Findings {
    let raws: Vec<&str> = session.entries().iter().map(Entry::raw).collect();
    let diagnosis = diagnose(NONE, &raws, &ENV, fs);
    Findings::of(session.entries(), diagnosis.scope(Scope::User))
}

#[test]
fn findings_are_read_by_entry_not_by_row() {
    let mut session = session(r"C:\tools;C:\gone");
    let findings = findings_over(&session, &Fs::new().directory(r"C:\tools"));
    let gone = session.entries()[1].id();

    // The flagged Entry moves to the top; nothing about it changed.
    assert!(session.move_up(gone));
    assert_eq!(findings.issues(&session.entries()[0]), &[Issue::Missing]);
    assert_eq!(findings.issues(&session.entries()[1]), &[] as &[Issue]);
}

#[test]
fn an_entry_whose_text_changed_has_no_findings_until_the_next_pass() {
    // The row NVDA reads after an edit must not be told what the *previous*
    // text was: a stale "Missing" on a path the user has just corrected is
    // worse than the empty column it will carry for one Timer tick.
    let mut session = session(r"C:\gone");
    let findings = findings_over(&session, &Fs::new());
    let id = session.entries()[0].id();
    assert_eq!(findings.issues(&session.entries()[0]), &[Issue::Missing]);

    assert!(session.edit(id, r"C:\tools"));
    assert_eq!(findings.issues(&session.entries()[0]), &[] as &[Issue]);
}

#[test]
fn an_entry_the_pass_never_saw_has_no_findings() {
    let mut session = session(r"C:\tools");
    let findings = findings_over(&session, &Fs::new().directory(r"C:\tools"));
    session.add(r"C:\gone");
    assert_eq!(findings.issues(&session.entries()[1]), &[] as &[Issue]);
}

#[test]
fn findings_hold_the_passs_own_issue_count() {
    // StatusBar field 0 reports the last pass, not the screen: the count does
    // not dip while an edited Entry waits for the next one (spec §12).
    let mut session = session(r#""C:\gone";C:\also-gone"#);
    let findings = findings_over(&session, &Fs::new());
    assert_eq!(findings.issue_count(), 3);

    let id = session.entries()[1].id();
    assert!(session.edit(id, r"C:\tools"));
    assert_eq!(findings.issue_count(), 3);
}

#[test]
fn no_pass_yet_means_no_findings_anywhere() {
    let session = session(r"C:\gone");
    let findings = Findings::default();
    assert_eq!(findings.issues(&session.entries()[0]), &[] as &[Issue]);
    assert_eq!(findings.issue_count(), 0);
}

#[test]
fn an_issue_type_is_a_catalogue_string() {
    // The Status column carries translated words, so every type must name a
    // msgid — and the five must not collide.
    let msgids: BTreeSet<&str> = Issue::SEVERITY
        .iter()
        .map(|issue| issue.catalogue_msgid())
        .collect();
    assert_eq!(msgids.len(), 5, "each Issue type names a distinct msgid");
    let registered: BTreeSet<&str> = msgids::REGISTRY.iter().map(|entry| entry.msgid).collect();
    for msgid in msgids {
        assert!(registered.contains(msgid), "{msgid:?} is in the Catalogue");
    }
}
