//! `CHANGELOG.md`, gated against the version Cargo carries.
//!
//! The file is hand-written and machine-read: the release workflow lifts a
//! version's section out of it and publishes that as the release page's body,
//! and Release Checklist step F2 renames `[Unreleased]` to the version being
//! released in the same commit that bumps `Cargo.toml` and the `.rc`. Both
//! halves fail quietly — a heading left behind publishes the wrong section, or
//! none at all — and neither is a compile error, so a test is the only place
//! the pairing can be held.
//!
//! It lives here, in the crate that links no wxWidgets, for the reason
//! `versioninfo.rs` gives for reaching outside its own crate: this is a
//! test-time path, not a dependency edge. The check is pure text over one file
//! read at compile time, so making it one of the binary's tests would tax it
//! with a wxWidgets link for nothing — and would cost ADR-0009 the claim that
//! the msgid smoke test "remains the only test that links wxWidgets"
//! (ADR-0007, ADR-0009).
//!
//! The version compared against is **this** crate's, which is the binary's
//! because both take `version.workspace = true` — the inheritance
//! `versioninfo.rs` asserts next door rather than assumes, and asserting it
//! twice would be one more thing to keep in step.
//!
//! **Green through development and red at F2, by construction.** `Cargo.toml`
//! says 0.1.0, the newest released heading is `[0.1.0]`, and `[Unreleased]`
//! grows above it for as long as the version stands still; the moment the
//! version moves and the heading does not, this fails.

/// The changelog, read at compile time: the tests need no filesystem and
/// cannot be defeated by being run from a different directory.
const CHANGELOG: &str = include_str!("../../../CHANGELOG.md");

/// The workspace version, which every crate here inherits.
const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Where the link references at the foot of the file must point.
///
/// Written out rather than derived from the file being checked, for the reason
/// `versioninfo.rs` gives about `CompanyName`: a link block lifted from
/// another project's changelog would be perfectly consistent with itself and
/// still point at somebody else's releases. That the same string also stands
/// in `pathmaster-platform`'s `help.rs`, carrying the User Guide's fallback
/// URL, is the point rather than a duplication to remove — one shared constant
/// would make the two agree by construction, and agreeing by construction is
/// the one thing a gate cannot check.
const REPOSITORY: &str = "https://github.com/ruslan-rv-ua/pathmaster";

/// The heading over the changes that have not been released yet — the one
/// heading that names no version, and the one F2 renames.
const UNRELEASED: &str = "Unreleased";

/// The file's version headings, in the order it lists them: newest first,
/// `[Unreleased]` above them all.
///
/// **This is the whole heading grammar, and the release workflow's note
/// extractor reads the same one** — a version heading is a line beginning
/// `## [`, and its version is the text up to the `]`. The extractor adds only
/// where a section *ends* (at the next such line, or at the end of the file).
/// One grammar read in two places: a second one would be a way for the test
/// and the release to disagree about which section a tag publishes.
///
/// Which is why a heading that never closes its `]` **panics** rather than
/// being passed over. The extractor ends a section at `## [` and never looks
/// for the `]` at all, so a line quietly dropped here would be a heading to
/// the release and not to the gate — precisely the disagreement the shared
/// grammar exists to prevent.
///
/// **Fenced code blocks are deliberately not skipped**, unlike the reader
/// `help.rs` keeps next door: a fence carrying a `## [` line would be a
/// heading to both readers, which is wrong together rather than wrong apart.
/// Buying the rule costs writing it twice, in Rust and in PowerShell, which is
/// the second grammar this one exists to avoid — and the entries here are
/// prose, so the file holds no fences to walk into.
fn version_headings(changelog: &'static str) -> impl Iterator<Item = &'static str> {
    changelog
        .lines()
        .filter_map(|line| line.strip_prefix("## ["))
        .map(|rest| {
            rest.split_once(']')
                .unwrap_or_else(|| {
                    panic!("CHANGELOG.md's heading `## [{rest}` never closes its `]`")
                })
                .0
        })
}

#[test]
fn the_reading_the_gate_is_built_on_sees_what_it_claims_to() {
    // Self-sensitivity: every assertion below is only as good as this
    // function, and what it must *not* match is invisible in a passing run.
    // The link references at the foot spell `[version]` too, a category
    // heading is a `###`, and prose can begin a line with `##`.
    let headings: Vec<&str> = version_headings(
        "# Changelog\n\
         \n\
         ## [Unreleased]\n\
         \n\
         ### Added\n\
         \n\
         - something\n\
         \n\
         ## [0.2.0] - 2026-09-01\n\
         \n\
         ## Not a version heading\n\
         \n\
         [0.2.0]: https://example.invalid/releases/tag/v0.2.0\n",
    )
    .collect();
    assert_eq!(headings, ["Unreleased", "0.2.0"]);
}

#[test]
#[should_panic(expected = "never closes its")]
fn a_heading_that_never_closes_its_bracket_fails_rather_than_being_skipped() {
    // The other half of the shared grammar, and the half the extractor cannot
    // see: it ends a section at `## [` without ever looking for the `]`, so
    // this line is a heading to the release whatever this reader does with it.
    // Passing over it here is how the two would come to disagree about where a
    // section starts, with nothing saying so.
    let _: Vec<&str> = version_headings(
        "## [Unreleased]

## [0.2.0 - 2026-09-01
",
    )
    .collect();
}

#[test]
fn the_newest_released_section_carries_the_crate_version() {
    // Read as a sequence rather than by searching, because the order is half
    // of what is being asserted: `[Unreleased]` is on top, and the heading
    // directly under it is the newest release — which is the one F2 wrote.
    let mut headings = version_headings(CHANGELOG);
    assert_eq!(
        headings.next(),
        Some(UNRELEASED),
        "CHANGELOG.md must open on its [{UNRELEASED}] section: F2 renames that heading and \
         opens a fresh empty one above it, and without it there is nowhere to write the next \
         change"
    );
    let newest = headings
        .next()
        .expect("CHANGELOG.md carries at least one released version heading");
    assert_eq!(
        newest, VERSION,
        "CHANGELOG.md's newest released section is [{newest}] while the crate is {VERSION} — \
         F2 bumps three files, and this is the third"
    );
}

#[test]
fn every_version_heading_carries_its_link_reference() {
    // The one maintenance point Keep a Changelog adds beyond the headings
    // themselves, and the one F2 touches twice: the released version gets a
    // reference of its own, *and* [Unreleased]'s compare base moves to it.
    // The second is what gets forgotten — it breaks nothing and shows up only
    // as a diff link quietly spanning two releases.
    let newest_released = version_headings(CHANGELOG)
        .find(|version| *version != UNRELEASED)
        .expect("CHANGELOG.md carries at least one released version heading");
    for version in version_headings(CHANGELOG) {
        let reference = if version == UNRELEASED {
            format!("[{UNRELEASED}]: {REPOSITORY}/compare/v{newest_released}...HEAD")
        } else {
            format!("[{version}]: {REPOSITORY}/releases/tag/v{version}")
        };
        assert!(
            CHANGELOG.lines().any(|line| line == reference),
            "CHANGELOG.md's [{version}] heading has no link reference at the foot of the file — \
             expected the line {reference:?}"
        );
    }
}
