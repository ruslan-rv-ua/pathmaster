//! The User Guide's two documents, gated against each other (v0.2.0 §9).
//!
//! `docs/help/en.md` and `docs/help/uk.md` are the source the build converts
//! into the pages the executable carries. Drift between them is the one defect
//! nothing else here would catch: a section added to the English guide and not
//! to the Ukrainian one leaves a Ukrainian reader a guide that is quietly
//! short, and no compiler, no `.po` gate and no Release Checklist step reads
//! both documents at once.
//!
//! It lives here, in the crate that links no wxWidgets, for the reason
//! `versioninfo.rs` gives: this is pure text over files read at test time, and
//! making it one of the binary's tests would tax it with a wxWidgets link for
//! nothing (ADR-0007, ADR-0009). Read at run time rather than `include_str!`-ed
//! because *whether the file exists* is one of the things being asserted.
//!
//! **Heading text is deliberately not compared.** The two documents are in two
//! languages, so no heading can read the same in both; what parity means here
//! is therefore structural — the same headings at the same levels in the same
//! order. That is exactly the drift worth gating, and it is all that can be.

use std::path::{Path, PathBuf};

use pathmaster_core::language::Language;

/// The languages that ship a guide — the same set that ships a catalogue,
/// because "one page per Interface Language" (v0.2.0 §9) is a claim about
/// this enum and not about whatever happens to be in the directory.
const LANGUAGES: [Language; 2] = [Language::English, Language::Ukrainian];

/// The documents live at the repository root, beside the rest of `docs/`, and
/// the failure rung of §9's ladder points a browser at exactly this path under
/// the version's own tag — so the location is part of the contract, not a
/// convenience.
fn help_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../docs/help")
}

fn document(language: Language) -> String {
    let path = help_dir().join(format!("{}.md", language.code()));
    std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("{} could not be read: {e}", path.display()))
}

/// One ATX heading: how deep it sits, and what it says.
#[derive(Debug, PartialEq, Eq)]
struct Heading {
    level: usize,
    text: String,
}

/// The document's headings, in the order a reader meets them.
///
/// Fenced code blocks are skipped, because a `# comment` inside a PowerShell
/// example is a comment and not a section — and the "Command line" subsection
/// (§9) is exactly where such a line would appear.
fn headings(markdown: &str) -> Vec<Heading> {
    let mut found = Vec::new();
    let mut fenced = false;
    for line in markdown.lines() {
        if line.trim_start().starts_with("```") {
            fenced = !fenced;
            continue;
        }
        if fenced {
            continue;
        }
        let hashes = line.len() - line.trim_start_matches('#').len();
        // An ATX heading is hashes then a space; `#word` is not one.
        if hashes > 0 && line[hashes..].starts_with(' ') {
            found.push(Heading {
                level: hashes,
                text: line[hashes..].trim().to_owned(),
            });
        }
    }
    found
}

#[test]
fn the_reading_the_gate_is_built_on_sees_what_it_claims_to() {
    // Self-sensitivity: every assertion below is only as good as this
    // function, and two of its rules are invisible in a passing run — a `#`
    // inside a fence is not a section, and `#word` is not a heading either.
    let headings = headings(
        "# Title\n\
         \n\
         ## A section\n\
         \n\
         ```\n\
         # not a heading, a comment\n\
         ```\n\
         \n\
         #hashtag\n\
         \n\
         ### A subsection\n",
    );
    assert_eq!(
        headings,
        [
            Heading {
                level: 1,
                text: "Title".to_owned()
            },
            Heading {
                level: 2,
                text: "A section".to_owned()
            },
            Heading {
                level: 3,
                text: "A subsection".to_owned()
            },
        ]
    );
}

#[test]
fn every_interface_language_ships_a_guide_with_something_in_it() {
    for language in LANGUAGES {
        let text = document(language);
        assert!(
            !text.trim().is_empty(),
            "docs/help/{}.md is empty",
            language.code()
        );
    }
}

#[test]
fn both_guides_carry_the_same_headings_at_the_same_levels() {
    // The comparison is over levels rather than text: two languages cannot
    // share a heading's words, and the shape is what says a section is
    // missing. Zipped rather than compared whole so the failure names the
    // first place they diverge instead of printing both documents.
    let english = headings(&document(Language::English));
    for language in LANGUAGES {
        let other = headings(&document(language));
        for (position, (mine, theirs)) in english.iter().zip(&other).enumerate() {
            assert_eq!(
                mine.level,
                theirs.level,
                "heading {position} differs in depth: en {:?} (level {}) against {} {:?} (level {})",
                mine.text,
                mine.level,
                language.code(),
                theirs.text,
                theirs.level,
            );
        }
        assert_eq!(
            english.len(),
            other.len(),
            "docs/help/{}.md carries {} headings against English's {}",
            language.code(),
            other.len(),
            english.len(),
        );
    }
}

#[test]
fn every_guide_opens_on_one_title_and_never_skips_a_level() {
    // The guide's whole navigation is the browser's heading list (§9), which
    // is a list of levels as much as of words: a document that jumps from `#`
    // to `###` reads to a screen reader as a section with a missing parent.
    for language in LANGUAGES {
        let headings = headings(&document(language));
        let (first, rest) = headings
            .split_first()
            .unwrap_or_else(|| panic!("docs/help/{}.md carries no heading", language.code()));
        assert_eq!(
            first.level,
            1,
            "docs/help/{}.md opens on {:?} at level {} rather than on one title",
            language.code(),
            first.text,
            first.level,
        );
        let mut previous = first.level;
        for heading in rest {
            assert!(
                heading.level <= previous + 1,
                "docs/help/{}.md jumps from level {previous} to {} at {:?}",
                language.code(),
                heading.level,
                heading.text,
            );
            previous = heading.level;
        }
    }
}
