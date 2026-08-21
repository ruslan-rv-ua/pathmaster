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
    HardCap,
}

impl Overlength {
    /// Whether an Apply may go ahead. The one place "the warning is walkable,
    /// the cap is not" is written down.
    pub fn may_proceed(self) -> bool {
        !matches!(self, Overlength::HardCap)
    }
}

/// The merged length of both Working Copies, in UTF-16 code units:
/// `len(expand(System WC) + ";" + expand(User WC))` — spec §7's formula, end
/// to end.
///
/// **Two callers ask this at different moments and must get one answer.** The
/// diagnostic pass asks after every edit, for the StatusBar; an Apply Run asks
/// again at the gate, because the pass's answer lags by a Timer tick and the
/// number in the dialog is the one the user is being asked to accept. Two
/// moments, one formula — which is why the formula is here rather than at
/// either of them.
pub fn merged_length_of(
    system: &[impl AsRef<str>],
    user: &[impl AsRef<str>],
    env: &dyn Environment,
) -> usize {
    merged_length(&expanded_value(system, env), &expanded_value(user, env))
}

/// The arithmetic half of [`merged_length_of`], over Scope values that are
/// already expanded — what Windows will materialise, not what the registry
/// stores. The separator counts even when a Scope is empty: the formula is the
/// spec's, taken literally.
pub fn merged_length(system_expanded: &str, user_expanded: &str) -> usize {
    utf16_len(system_expanded) + 1 + utf16_len(user_expanded)
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
