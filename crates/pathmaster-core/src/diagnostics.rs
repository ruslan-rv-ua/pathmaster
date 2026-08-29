//! The diagnostic rulebook: spec §7's six Issue types over the two Working
//! Copies — the five that name an Entry, here, and the scope-level over-length,
//! in [`crate::thresholds`]. CONTEXT.md's Issue covers findings "about one
//! Entry **or about a Scope as a whole**", and the split follows that seam: an
//! over-length has no Entry to name, never enters the Status column, and asks a
//! question none of the five do ("may this Apply proceed?").
//!
//! Issues are a **derived view**. Nothing here writes into a Working Copy, and
//! nothing here is captured in a Checkpoint — a pass is recomputed after every
//! edit, undo, Refresh and Restore, so an undo can never reinstate a diagnosis
//! of a state no longer on screen (ADR-0001).
//!
//! Everything the rules cannot answer from text alone is injected: the process
//! environment ([`Environment`], for `%VAR%`) and the filesystem
//! ([`Filesystem`], for "does this name a directory?"). Core takes no OS call,
//! and a rulebook whose answers depend on the machine it runs on cannot be
//! tested. The Windows adapters and the worker thread that drives a pass land
//! with ticket 12; this module is the whole of what they will run.
//!
//! **Evaluation order is runtime order** — System Working Copy first, then
//! User, each left to right — because that is the order Windows merges the two
//! Scopes in, and it is what decides which copy of a duplicate is the one that
//! actually wins. Hence one pass over both Scopes rather than one per Scope:
//! a System edit changes User's findings.

use std::collections::HashSet;

use crate::normalize::{expand, strip_quotes, Environment, Expansion, Normalised};
use crate::session::{Entry, EntryId, Scope};
use crate::thresholds::{self, Overlength};

/// One diagnostic finding about one Entry (spec §7) — five of the six types.
///
/// The sixth, over-length, is deliberately not here: no Entry is at fault for
/// a length that only exists once both Scopes are merged, and D6 takes it out
/// of the per-entry set entirely (see [`crate::thresholds`]).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Issue {
    /// The path does not name an existing directory.
    Missing,
    /// The Entry is not fully qualified, so what it names depends on the
    /// process's current state.
    Relative,
    /// The Entry contains a `"`. Measured: the quoted spelling is dead for
    /// `CreateProcessW`, `SearchPathW`, PowerShell, `where` and Python, and
    /// alive for cmd, the CRT, Rust and Node — silent breakage, trivial fix.
    Quoted,
    /// An earlier Entry — in this Scope or in System — normalises the same.
    Duplicate,
    /// Zero-length or whitespace-only: no usable path text.
    Empty,
}

impl Issue {
    /// The five types most-severe-first — the order the Status column joins
    /// them in, and the single source of that order (spec §7, FR-diag-status).
    pub const SEVERITY: [Issue; 5] = [
        Issue::Missing,
        Issue::Relative,
        Issue::Quoted,
        Issue::Duplicate,
        Issue::Empty,
    ];

    /// The Catalogue string the Status column shows for this type. One word,
    /// and never a severity prefix or an icon (spec §7, FR-diag-status).
    pub fn catalogue_msgid(&self) -> &'static str {
        match self {
            Issue::Missing => crate::msgids::ISSUE_MISSING,
            Issue::Relative => crate::msgids::ISSUE_RELATIVE,
            Issue::Quoted => crate::msgids::ISSUE_QUOTED,
            Issue::Duplicate => crate::msgids::ISSUE_DUPLICATE,
            Issue::Empty => crate::msgids::ISSUE_EMPTY,
        }
    }
}

/// Where a path's root lives.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RootKind {
    /// A local disk — checking it costs microseconds.
    Local,
    /// A network root: a UNC path or a mapped remote drive. Never probed in
    /// v0.1.0 (a dead one blocks 20-60 s and cannot be cancelled).
    Network,
}

/// What one probe of a path found.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Existence {
    /// An existing directory — the only healthy answer.
    Directory,
    /// An existing file. Inert in a `PATH`: the search appends `\name.exe` to
    /// the Entry-as-directory, so a file Entry finds nothing.
    File,
    /// Nothing of that name.
    NotFound,
    /// It exists but cannot be read. Calling this missing is the long-standing
    /// `File.Exists` mistake, and this rulebook does not make it.
    AccessDenied,
}

/// The filesystem facts the rules cannot answer for themselves.
///
/// Two questions, deliberately separate: the root is classified without a
/// network round trip (`GetDriveTypeW`, the UNC prefix), and only a local root
/// is ever probed.
pub trait Filesystem {
    /// Where `path`'s root lives. Must not touch the network.
    fn root_kind(&self, path: &str) -> RootKind;

    /// What `path` names. Called only for local roots.
    fn probe(&self, path: &str) -> Existence;
}

/// One Working Copy's Issues: one list per Entry, in list order, each list
/// most-severe-first. An empty list is the only healthy state — never "OK".
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ScopeDiagnosis {
    entries: Vec<Vec<Issue>>,
}

impl ScopeDiagnosis {
    /// The Issues of the Entry at `index`.
    ///
    /// An index past the end has none: a pass describes the Working Copy it
    /// ran over, which a later edit may already have shortened, and answering
    /// beats panicking in a window NVDA is reading.
    pub fn issues(&self, index: usize) -> &[Issue] {
        self.entries.get(index).map_or(&[], Vec::as_slice)
    }

    /// How many Entries this pass covered.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Every finding in this Scope — three on one Entry counts three.
    pub fn issue_count(&self) -> usize {
        self.entries.iter().map(Vec::len).sum()
    }
}

/// What one Scope's Status column reads: the last completed pass, held against
/// a Working Copy that may already have moved on, plus whatever
/// [`stamp`](Findings::stamp) has recorded since (spec §7, FR-diag-async;
/// ADR-0013).
///
/// A [`ScopeDiagnosis`] is indexed by row, and a row is exactly what an edit
/// changes. So the pass is kept **by Entry id, beside the text it ran over**,
/// and both must match before a finding is shown: an Entry that only moved
/// keeps its Issues, and an Entry whose text has changed carries none until
/// the new pass lands. The alternative — reading the old pass by row — would
/// put a stale "Missing" on the path the user has just corrected, in the one
/// window where focus is landing on it and NVDA is reading it aloud.
///
/// Nothing here is ever written back into a Working Copy or a Checkpoint: this
/// is the derived view, held for the length of one Timer tick (ADR-0001).
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Findings {
    entries: Vec<Finding>,
}

/// What the pass found about one Entry, and the two facts that say whether it
/// still describes it.
#[derive(Debug, Clone, PartialEq, Eq)]
struct Finding {
    id: EntryId,
    raw: String,
    issues: Vec<Issue>,
}

impl Findings {
    /// Pairs a completed pass with the Entries it ran over, in the order it ran
    /// over them. A pass and a Working Copy of different lengths can only be a
    /// pass that has already been overtaken; the shorter of the two wins, and
    /// the Entries past it read as unseen.
    pub fn of(entries: &[Entry], diagnosis: &ScopeDiagnosis) -> Findings {
        Findings {
            entries: entries
                .iter()
                .take(diagnosis.len())
                .enumerate()
                .map(|(index, entry)| Finding {
                    id: entry.id(),
                    raw: entry.raw().to_string(),
                    issues: diagnosis.issues(index).to_vec(),
                })
                .collect(),
        }
    }

    /// What the last pass found about `entry` — and nothing at all for an Entry
    /// it never saw, or one whose text has changed since it ran.
    pub fn issues(&self, entry: &Entry) -> &[Issue] {
        self.entries
            .iter()
            .find(|finding| finding.id == entry.id())
            .filter(|finding| finding.raw == entry.raw())
            .map_or(&[], |finding| &finding.issues)
    }

    /// Records one Entry's Issues without a pass, so that the Entry an Add or
    /// an Edit just gave new text is not Undiagnosed in the one window where
    /// focus lands on its row and NVDA reads it (ADR-0013).
    ///
    /// Keyed by `(id, raw)` like everything else here, so the ordinary staleness
    /// rule applies to a stamp exactly as it applies to a pass: edit the Entry
    /// again and the stamp stops describing it, in the same breath and for the
    /// same reason. An Entry already recorded is replaced in place; a new one —
    /// Add's, appended to the Working Copy — goes on the end, which is where it
    /// sits in the list.
    ///
    /// A stamp is worth nothing unless it agrees with the pass that will
    /// overwrite it, which is why the caller has no say in what it says: the
    /// Issues handed here come from [`diagnose_one`], which is the pass's own
    /// rulebook answering about one Entry.
    pub fn stamp(&mut self, id: EntryId, raw: &str, issues: Vec<Issue>) {
        let stamped = Finding {
            id,
            raw: raw.to_string(),
            issues,
        };
        match self.entries.iter_mut().find(|finding| finding.id == id) {
            Some(held) => *held = stamped,
            None => self.entries.push(stamped),
        }
    }

    /// Every finding this Scope carries — three on one Entry counts three.
    ///
    /// The last pass's, plus whatever has been [`stamped`](Self::stamp) since:
    /// StatusBar field 0 is updated after every pass (spec §12) and after every
    /// edit, so an Entry edited since a pass and not stamped still counts here
    /// while its column waits for the next one.
    pub fn issue_count(&self) -> usize {
        self.entries
            .iter()
            .map(|finding| finding.issues.len())
            .sum()
    }
}

/// One pass's results: both Scopes' Issues and the merged length.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Diagnosis {
    system: ScopeDiagnosis,
    user: ScopeDiagnosis,
    merged_length: usize,
}

impl Diagnosis {
    /// One Scope's findings.
    pub fn scope(&self, scope: Scope) -> &ScopeDiagnosis {
        match scope {
            Scope::System => &self.system,
            Scope::User => &self.user,
        }
    }

    /// `len(expand(System WC) + ";" + expand(User WC))` in UTF-16 code units —
    /// what the StatusBar shows always (spec §12).
    pub fn merged_length(&self) -> usize {
        self.merged_length
    }

    /// Which side of the two thresholds that length falls (spec §7).
    pub fn overlength(&self) -> Overlength {
        thresholds::classify(self.merged_length)
    }
}

/// Runs one diagnostic pass over both Working Copies.
///
/// The argument order is the runtime order — **System first** — and it is what
/// decides which copy of a cross-scope duplicate carries the flag: the first
/// occurrence is canonical and clean, so the User copy is the one that reads
/// `Duplicate`.
pub fn diagnose(
    system: &[impl AsRef<str>],
    user: &[impl AsRef<str>],
    env: &dyn Environment,
    fs: &dyn Filesystem,
) -> Diagnosis {
    let mut seen = HashSet::new();
    let mut diagnosed = (Vec::new(), Vec::new());
    for (scope, _, raw) in in_runtime_order(system, user) {
        let issues = diagnose_entry(raw, env, fs, &mut seen);
        match scope {
            Scope::System => diagnosed.0.push(issues),
            Scope::User => diagnosed.1.push(issues),
        }
    }
    Diagnosis {
        system: ScopeDiagnosis {
            entries: diagnosed.0,
        },
        user: ScopeDiagnosis {
            entries: diagnosed.1,
        },
        merged_length: thresholds::merged_length(system, user, env),
    }
}

/// The Issues of the Entry at `index` of `scope`'s Working Copy — the answer a
/// whole [`diagnose`] pass would give about that one Entry, at the cost of
/// probing that one Entry (ADR-0013).
///
/// It exists because a pass is asynchronous and an Add or an Edit is not: the
/// Entry whose text has just changed is the one row focus is about to land on,
/// and the last completed pass cannot describe it. Everything the pass would
/// have to say about *other* Entries is deliberately not computed here.
///
/// The Entries before it are read for their duplicate keys alone — **none of
/// them is probed**, so the filesystem is asked exactly one question. Which
/// Entries those are is decided here rather than by the caller: it is the
/// runtime order, and a caller free to pass the wrong prefix is a caller free
/// to move a cross-scope duplicate onto the wrong Scope.
///
/// An `index` past the end of that Scope's Working Copy has no Issues, for
/// [`ScopeDiagnosis::issues`]' reason: answering beats panicking in a window
/// NVDA is reading.
pub fn diagnose_one(
    system: &[impl AsRef<str>],
    user: &[impl AsRef<str>],
    scope: Scope,
    index: usize,
    env: &dyn Environment,
    fs: &dyn Filesystem,
) -> Vec<Issue> {
    let mut seen = HashSet::new();
    for (at, position, raw) in in_runtime_order(system, user) {
        if at == scope && position == index {
            return diagnose_entry(raw, env, fs, &mut seen);
        }
        if let Some(expansion) = reading(raw, env) {
            seen.insert(key(&expansion));
        }
    }
    Vec::new()
}

/// Both Working Copies as one walk, in the order Windows searches them —
/// **System first** — each Entry carrying the Scope it belongs to and its
/// position within that Scope.
///
/// One home for that order, and the reason it needs one: it is what decides
/// which copy of a cross-scope duplicate carries the flag, so a second reading
/// of it would move every such flag onto the other Scope with nothing failing
/// to compile.
fn in_runtime_order<'a>(
    system: &'a [impl AsRef<str>],
    user: &'a [impl AsRef<str>],
) -> impl Iterator<Item = (Scope, usize, &'a str)> {
    let system = system
        .iter()
        .enumerate()
        .map(|(position, entry)| (Scope::System, position, entry.as_ref()));
    let user = user
        .iter()
        .enumerate()
        .map(|(position, entry)| (Scope::User, position, entry.as_ref()));
    system.chain(user)
}

/// The rules for one Entry, and their coexistence: Empty is exclusive,
/// Relative and Missing never co-occur, Quoted co-occurs freely.
fn diagnose_entry(
    raw: &str,
    env: &dyn Environment,
    fs: &dyn Filesystem,
    seen: &mut HashSet<Normalised>,
) -> Vec<Issue> {
    // Empty is exclusive, and an Empty Entry has no reading at all.
    let Some(expansion) = reading(raw, env) else {
        return vec![Issue::Empty];
    };
    let path = expansion.text.as_str();

    let mut flagged = Vec::new();
    if raw.contains('"') {
        flagged.push(Issue::Quoted);
    }
    if is_probed(&expansion) {
        if is_missing(path, fs) {
            flagged.push(Issue::Missing);
        }
    } else {
        flagged.push(Issue::Relative);
    }
    if !seen.insert(key(&expansion)) {
        flagged.push(Issue::Duplicate);
    }

    // Most-severe-first, and in one place: the Status column joins this list as
    // it stands, so the order is the declared one rather than the order the
    // checks happened to run in.
    Issue::SEVERITY
        .iter()
        .filter(|issue| flagged.contains(issue))
        .copied()
        .collect()
}

/// One Entry's reading — quotes stripped, `%VAR%` expanded — or `None` for an
/// Entry that has none.
///
/// The single home of a rule two callers need and neither may spell twice: an
/// Entry that is zero-length or whitespace-only is no path at all, so it is
/// never probed and it puts **no key** into the duplicate set — which is why
/// two Empty Entries are not duplicates of each other.
fn reading(raw: &str, env: &dyn Environment) -> Option<Expansion> {
    if raw.trim().is_empty() {
        return None;
    }
    Some(expand(strip_quotes(raw), env))
}

/// The key a reading puts into the duplicate set. Spelled once, because it is
/// the whole of "are these two the same path?".
fn key(expansion: &Expansion) -> Normalised {
    Normalised::of_expanded(&expansion.text)
}

/// Whether the existence check runs over a reading, which is the same question
/// as whether its shape is legible at all.
///
/// Text that *begins* with an unresolved `%VAR%` is not judged for shape: what
/// the path would have started with is exactly what is missing, and the spec
/// sends that case to the existence check, where the literal text fails
/// (FR-diag-missing, D10). A reference further along leaves the shape legible,
/// so `tools\%NOPE%` is Relative like any other bare name.
///
/// One home, two readers: the rule above, and [`probe_target`], which exists
/// so a caller can find out whether a probe is coming **before** paying for
/// it. Those two may not come to disagree.
fn is_probed(expansion: &Expansion) -> bool {
    expansion.starts_unresolved || is_fully_qualified(&expansion.text)
}

/// The text the existence check would probe for `raw`, or `None` for an Entry
/// it never reaches — an Empty one, or one whose shape is already Relative.
///
/// The question a caller asks when it must decide whether diagnosing this
/// Entry is affordable *here* — on a thread that must not block, over a root
/// that may not be there (ADR-0013). It answers what would be probed, never
/// whether probing it is wise: that is a fact about the machine, and the
/// rulebook does not hold those.
pub fn probe_target(raw: &str, env: &dyn Environment) -> Option<String> {
    let expansion = reading(raw, env)?;
    is_probed(&expansion).then_some(expansion.text)
}

/// The existence check: local roots only, directories only.
fn is_missing(path: &str, fs: &dyn Filesystem) -> bool {
    if fs.root_kind(path) == RootKind::Network {
        return false;
    }
    match fs.probe(path) {
        Existence::Directory | Existence::AccessDenied => false,
        Existence::File | Existence::NotFound => true,
    }
}

/// Fully qualified: `X:\…`, `\\server\share…`, `\\?\…` — a path that names the
/// same directory whatever the process's current drive and directory are.
/// Everything else resolves against process state: `.`, `..`, bare names,
/// rooted `\foo`, drive-relative `C:foo` (the .NET path taxonomy; Win32's own
/// `PathIsRelativeW` passes both of the hazardous last two).
///
/// Either separator qualifies — `C:/foo` and `//server/share` are the same
/// paths to Win32 as their backslash spellings.
///
/// Public because the Tree View asks the same question of the same text: an
/// Entry with no filesystem position gets a group node instead of a place
/// under a drive root (v0.2.0 §6). One rule, one home — a second reading of
/// "can this path be placed?" is a second answer waiting to disagree with the
/// `Relative` an Entry's Status column already carries.
pub fn is_fully_qualified(path: &str) -> bool {
    let mut chars = path.chars();
    match (chars.next(), chars.next(), chars.next()) {
        (Some(first), Some(second), _) if is_separator(first) && is_separator(second) => true,
        (Some(drive), Some(':'), Some(third)) if drive.is_ascii_alphabetic() => is_separator(third),
        _ => false,
    }
}

fn is_separator(c: char) -> bool {
    c == '\\' || c == '/'
}
