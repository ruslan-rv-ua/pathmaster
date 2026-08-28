//! The User Guide on disk: the fourth file in the Data Directory, and the
//! address of the copy that stands in when it cannot be written (v0.2.0 §9).
//!
//! The page itself is not here. It is compiled from `docs/help/<code>.md` at
//! build time and carried in the executable's own bytes, exactly as the
//! catalogues are (NFR-portable), so what this module takes is bytes and what
//! it answers is where they landed. That split is what lets the file's rules
//! be tested without a wxWidgets link and without the binary's `OUT_DIR`.
//!
//! **Written unconditionally, every time.** "Write only if missing" is poisoned
//! here: scoop persists `data\` as a junction across upgrades, so a v0.2.0
//! binary would show v0.1.0's guide forever and nothing would ever say so.
//! Rewriting is cheap, atomic, and makes staleness structurally impossible.

use std::io;
use std::path::{Path, PathBuf};

use crate::datadir;

/// The page's name in the Data Directory — **one file, no language suffix**.
///
/// A per-language name would leave an orphan behind the first time the
/// Interface Language changed, and the orphan would be a guide: still readable,
/// still findable, and wrong. Never translated — a file name is outside the
/// Catalogue (spec §11).
pub const FILE_NAME: &str = "help.html";

/// Where the guide is written for a Run whose Data Directory is `data_dir`.
pub fn page_path(data_dir: &Path) -> PathBuf {
    data_dir.join(FILE_NAME)
}

/// Writes the page and answers where it went.
///
/// Atomic, through [`datadir::write_replace`]: the bytes land in a pid-unique
/// `.tmp` beside the target and replace it in one rename, so the browser — or
/// the other instance, which is a designed state — never opens half a file.
///
/// The failure this can answer with is the ladder's second rung, not an error
/// to report: Read-only Data and a full disk both arrive here, and what the
/// caller does with them is open the online copy instead.
pub fn write_page(data_dir: &Path, page: &[u8]) -> io::Result<PathBuf> {
    let target = page_path(data_dir);
    datadir::write_replace(&target, page)?;
    Ok(target)
}

/// The address of the source document for `version`, in `language_code` — the
/// rung below the file (v0.2.0 §9).
///
/// **Version-pinned, never `main`.** A guide that describes a build the reader
/// is not running is worse than no guide, and `blob/v{version}/` is the one
/// address that cannot drift: the tag it names is the build that opened it.
/// The consequence is named rather than fixed — in a development build the URL
/// 404s until the tag exists, and the Release Checklist runs on a tagged build.
///
/// The Markdown source rather than a rendered page, because it is what the
/// repository actually holds; GitHub renders it on arrival, headings and all.
pub fn source_url(version: &str, language_code: &str) -> String {
    format!("{REPOSITORY}/blob/v{version}/docs/help/{language_code}.md")
}

/// The repository the release is cut from — the same one the README's badges
/// and the Issues link name.
const REPOSITORY: &str = "https://github.com/ruslan-rv-ua/pathmaster";
