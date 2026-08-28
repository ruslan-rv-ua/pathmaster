//! The User Guide at runtime: the embedded pages, and the one lookup F1 goes
//! through (v0.2.0 §9).
//!
//! Nothing is read from disk. The pages are `include_bytes!`-ed from `OUT_DIR`,
//! where `build.rs` converted them from `docs/help/*.md`, so a single file
//! stays a single file (NFR-portable) — the same mechanism, for the same
//! reason, as the Catalogue's `.mo` files next door.
//!
//! What is *done* with the bytes is not here: writing them into the Data
//! Directory and handing the result to the browser is
//! [`pathmaster_platform::help`], which owns the filesystem and the shell. This
//! module owns only the question "which page", because that is the only part
//! that needs the executable's own bytes.

use pathmaster_core::language::Language;

// `static HELP_PAGES: &[(&str, &[u8])]` — one row per `docs/help/<code>.md`.
include!(concat!(env!("OUT_DIR"), "/help_pages.rs"));

/// The page `language` reads.
///
/// A language with no page of its own takes the first, which is English: the
/// Catalogue's own fallback rule (a lookup that finds nothing returns the
/// msgid, and the msgid is English) applied to the document. `build.rs` makes
/// that unreachable — it refuses to build a language that ships a catalogue and
/// no page — so this is the net under the gate rather than the design.
pub fn page(language: Language) -> &'static [u8] {
    let (_, page) = HELP_PAGES
        .iter()
        .find(|(code, _)| *code == language.code())
        .or_else(|| HELP_PAGES.first())
        .expect("build.rs writes at least one page");
    page
}
