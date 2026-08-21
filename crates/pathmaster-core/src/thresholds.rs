//! How long a `PATH` may be, and what happens past each number (spec §7,
//! FR-diag-overlength).
//!
//! Over-length is scope-level and never per-entry: no Entry is at fault for a
//! length that only exists once both Scopes are merged, so it never enters the
//! Status column and is never an Announcement. It surfaces twice — the
//! StatusBar's length field, always, and an Apply-time dialog past a threshold.
//!
//! Two numbers, measured rather than inherited (research/13). Both are counted
//! in UTF-16 code units, because that is the unit Windows stores a value in.

use crate::normalize::{expand, Environment};
use crate::path::join;

/// Past this, `cmd.exe` ignores the inherited variable entirely — the `PATH`
/// is simply absent inside a command prompt (KB 830473). A real consequence,
/// and a legal thing for a user to choose knowingly.
pub const CMD_LIMIT: usize = 8_191;

/// At this, the value cannot be materialised into any process environment
/// (`SetEnvironmentVariableW`). Nothing can be offered but Cancel.
pub const HARD_CAP: usize = 32_767;

/// What a merged length means.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Overlength {
    /// Under every threshold. No dialog, nothing to say.
    Within,
    /// Past [`CMD_LIMIT`]: a warning dialog with a proceed button.
    CmdLimit,
    /// At or past [`HARD_CAP`]: a dialog with no proceed button.
    ///
    /// "The warning is walkable, the cap is not" was once a `may_proceed`
    /// predicate here. It had no caller and could not honestly get one: an
    /// Apply Run does three different things with these three variants — say
    /// nothing, ask, tell — and a `bool` collapses two of them. The rule is
    /// stated where it is now also *enforced*, on the Apply port's `hard_cap`,
    /// which has no answer to give.
    HardCap,
}

/// The merged length of both Working Copies, in UTF-16 code units:
/// `len(expand(System WC) + ";" + expand(User WC))` — spec §7's formula, whole,
/// and the only way to ask for it.
///
/// **Two callers ask at different moments and must get one answer.** The
/// diagnostic pass asks after every edit, for the StatusBar; an Apply Run asks
/// again at its gate, because the pass's answer lags by a Timer tick and the
/// number in the dialog is the one the user is being asked to accept. Two
/// moments, one formula — which is why the formula is here rather than at
/// either of them, and why the expansion is not a step a caller can perform on
/// its own and hand in half-done.
///
/// The separator counts even when a Scope is empty: the formula is the spec's,
/// taken literally.
pub fn merged_length(
    system: &[impl AsRef<str>],
    user: &[impl AsRef<str>],
    env: &dyn Environment,
) -> usize {
    utf16_len(&expanded_value(system, env)) + 1 + utf16_len(&expanded_value(user, env))
}

/// One Scope's Working Copy as Windows will materialise it: the Entries joined
/// with `;`, expanded once.
fn expanded_value(entries: &[impl AsRef<str>], env: &dyn Environment) -> String {
    expand(&join(entries), env).text
}

/// Which side of the two thresholds a length falls.
pub fn classify(length: usize) -> Overlength {
    if length >= HARD_CAP {
        Overlength::HardCap
    } else if length > CMD_LIMIT {
        Overlength::CmdLimit
    } else {
        Overlength::Within
    }
}

fn utf16_len(text: &str) -> usize {
    text.chars().map(char::len_utf16).sum()
}
