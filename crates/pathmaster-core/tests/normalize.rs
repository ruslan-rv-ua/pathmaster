//! Normalisation at the crate boundary (spec §7 FR-diag-normalise, ticket impl-09).
//!
//! Comparison-only: nothing here is stored, written, or asked of the filesystem.
//! The pipeline is fixed — strip one pair of surrounding `"` → expand `%VAR%`
//! (unknown names stay literal) → `/`→`\` → trim trailing `\` unless that leaves
//! a bare root → compare ordinal case-insensitively.

use pathmaster_core::normalize::{expand, strip_quotes, Environment, Normalised};

/// A fixed environment, looked up case-insensitively as Windows' own is.
struct Env(&'static [(&'static str, &'static str)]);

impl Environment for Env {
    fn lookup(&self, name: &str) -> Option<String> {
        self.0
            .iter()
            .find(|(key, _)| key.eq_ignore_ascii_case(name))
            .map(|(_, value)| (*value).to_string())
    }
}

/// Plain values only: a variable whose value carries a reference of its own
/// would break idempotence, which is exactly what `SELF_REFERENTIAL` shows.
const ENV: Env = Env(&[("SystemRoot", r"C:\Windows"), ("JAVA_HOME", r"C:\Java")]);

const SELF_REFERENTIAL: Env = Env(&[("REF", r"%SystemRoot%\sub"), ("SystemRoot", r"C:\Windows")]);

fn normalised(entry: &str) -> String {
    Normalised::of(entry, &ENV).as_str().to_string()
}

fn same(left: &str, right: &str) -> bool {
    Normalised::of(left, &ENV) == Normalised::of(right, &ENV)
}

// ---- Step 1: one pair of surrounding quotes ----

#[test]
fn one_pair_of_surrounding_quotes_is_stripped() {
    assert_eq!(
        strip_quotes(r#""C:\Program Files\foo""#),
        r"C:\Program Files\foo"
    );
}

#[test]
fn only_one_pair_comes_off() {
    assert_eq!(strip_quotes(r#"""C:\foo"""#), r#""C:\foo""#);
}

#[test]
fn an_unpaired_quote_is_left_alone() {
    assert_eq!(strip_quotes(r#""C:\foo"#), r#""C:\foo"#);
    assert_eq!(strip_quotes(r#"C:\foo""#), r#"C:\foo""#);
    assert_eq!(strip_quotes(r#"""#), r#"""#);
    assert_eq!(strip_quotes(r#"C:\a"b\c"#), r#"C:\a"b\c"#);
}

#[test]
fn a_quoted_entry_normalises_onto_its_bare_spelling() {
    assert!(same(r#""C:\Program Files\foo""#, r"C:\Program Files\foo"));
}

// ---- Step 2: `%VAR%` expansion, ExpandEnvironmentStringsW's own semantics ----

#[test]
fn a_defined_reference_expands_and_the_lookup_ignores_case() {
    assert_eq!(expand("%SystemRoot%", &ENV).text, r"C:\Windows");
    assert_eq!(expand("%SYSTEMROOT%", &ENV).text, r"C:\Windows");
    assert_eq!(
        expand(r"%systemroot%\System32", &ENV).text,
        r"C:\Windows\System32"
    );
    assert!(!expand("%SystemRoot%", &ENV).starts_unresolved);
}

#[test]
fn an_unknown_name_stays_literal() {
    let expansion = expand(r"%NOPE%\bin", &ENV);
    assert_eq!(expansion.text, r"%NOPE%\bin");
}

#[test]
fn only_an_unknown_name_in_the_leading_position_is_reported() {
    // What a path starts with is what decides its shape, so that is the one
    // position an unresolved reference makes unanswerable.
    assert!(expand(r"%NOPE%\bin", &ENV).starts_unresolved);
    assert!(expand("%NOPE%", &ENV).starts_unresolved);
    assert!(!expand(r"tools\%NOPE%", &ENV).starts_unresolved);
    assert!(!expand(r"C:\%NOPE%", &ENV).starts_unresolved);
    assert!(!expand(r"%SystemRoot%\%NOPE%", &ENV).starts_unresolved);
}

#[test]
fn a_failed_reference_hands_its_closing_percent_back_to_the_scan() {
    // Measured against ExpandEnvironmentStringsW: a failed lookup emits the
    // opening `%` and rescans from the next character, so the `%` that closed
    // it can open the next reference.
    assert_eq!(expand("%NOPE%SystemRoot%", &ENV).text, r"%NOPEC:\Windows");
    assert_eq!(
        expand("a%NOPE%b%SystemRoot%c", &ENV).text,
        r"a%NOPE%bC:\Windowsc"
    );
}

#[test]
fn an_empty_name_is_not_a_reference() {
    // `%%` expands to nothing and resolves nothing — measured `%%SystemRoot%%`
    // → `%C:\Windows%`.
    let expansion = expand("%%SystemRoot%%", &ENV);
    assert_eq!(expansion.text, r"%C:\Windows%");
    assert!(!expansion.starts_unresolved);
    assert_eq!(expand("x%%y", &ENV).text, "x%%y");
}

#[test]
fn an_unterminated_reference_stays_literal() {
    assert_eq!(expand("%SystemRoot", &ENV).text, "%SystemRoot");
    assert_eq!(expand("%", &ENV).text, "%");
    assert_eq!(expand(r"C:\100%\bin", &ENV).text, r"C:\100%\bin");
    assert!(!expand(r"C:\100%\bin", &ENV).starts_unresolved);
}

#[test]
fn expansion_is_a_single_pass() {
    // As ExpandEnvironmentStringsW is: a value carrying a reference is not
    // expanded again.
    assert_eq!(expand("%REF%", &SELF_REFERENTIAL).text, r"%SystemRoot%\sub");
}

#[test]
fn a_reference_and_its_expansion_normalise_alike() {
    assert!(same(r"%SystemRoot%\System32", r"C:\Windows\System32"));
    assert!(same(r"%JAVA_HOME%\bin", r"c:\java\BIN\"));
}

// ---- Step 3: slash direction ----

#[test]
fn forward_slashes_become_backslashes() {
    assert_eq!(normalised("C:/Windows/System32"), r"C:\WINDOWS\SYSTEM32");
    assert!(same("C:/Windows", r"C:\Windows"));
}

// ---- Step 4: the trailing separator, and the bare root that keeps it ----

#[test]
fn a_trailing_separator_is_trimmed() {
    assert!(same(r"C:\Windows\", r"C:\Windows"));
    assert_eq!(normalised(r"C:\Windows\\\"), r"C:\WINDOWS");
    assert!(same(r"\\server\share\", r"\\server\share"));
}

#[test]
fn a_bare_root_keeps_its_separator() {
    // `C:\` trimmed to `C:` would name the current directory on C:, which is
    // a different place.
    assert_eq!(normalised(r"C:\"), r"C:\");
    assert_eq!(normalised(r"C:\\"), r"C:\");
    assert_eq!(normalised(r"\"), r"\");
    assert!(!same(r"C:\", "C:"));
}

// ---- Step 5: ordinal case-insensitive comparison ----

#[test]
fn comparison_ignores_case() {
    assert!(same(r"C:\Windows\System32", r"c:\windows\system32"));
    assert!(same(r"C:\ПРОГРАМИ", r"c:\програми"));
}

#[test]
fn different_paths_stay_different() {
    assert!(!same(r"C:\Windows", r"C:\Windows\System32"));
    assert!(!same(r"C:\one", r"C:\two"));
    assert!(!same(r"C:\Windows", r"D:\Windows"));
}

// ---- The whole pipeline ----

#[test]
fn the_pipeline_runs_in_the_specified_order() {
    // Quotes off, then expansion, then slashes, then the trailing separator,
    // then the fold — one Entry exercising every step at once.
    assert_eq!(
        normalised(r#""%SystemRoot%/System32/""#),
        r"C:\WINDOWS\SYSTEM32"
    );
}

#[test]
fn whitespace_is_part_of_the_entry_and_survives() {
    // Normalisation has five steps and trimming is not one of them: a leading
    // space is a different Entry, not a duplicate.
    assert!(!same(r" C:\Windows", r"C:\Windows"));
}

proptest::proptest! {
    #[test]
    fn normalisation_is_idempotent(entry in r#"[a-zA-Z0-9%/\\:. "]{0,32}"#) {
        let once = Normalised::of(&entry, &ENV);
        // Quote stripping takes exactly one pair by spec, so a doubly quoted
        // Entry loses a pair per pass — the one shape a second pass moves, and
        // one no pass ever produces.
        proptest::prop_assume!(
            !(once.as_str().len() > 1
                && once.as_str().starts_with('"')
                && once.as_str().ends_with('"'))
        );
        let twice = Normalised::of(once.as_str(), &ENV);
        proptest::prop_assert_eq!(twice, once);
    }
}
