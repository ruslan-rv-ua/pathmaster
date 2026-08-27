//! The Data Directory: located from the executable, never relocated
//! (spec §3, ADR-0002) — unless a single Run was pointed elsewhere by
//! `--data-dir`, which substitutes the locate step and nothing else
//! (v0.2.0 §10).

use std::ffi::{OsStr, OsString};
use std::fs;
use std::io;
use std::os::windows::ffi::{OsStrExt, OsStringExt};
use std::path::{Component, Path, PathBuf, Prefix, PrefixComponent};

use pathmaster_core::logfmt;
use pathmaster_core::msgids;
use windows_sys::Win32::Storage::FileSystem::{MoveFileExW, MOVEFILE_REPLACE_EXISTING};

/// Why a run is in Read-only Data — exactly these four (spec §3; v0.2.0 §10).
/// Each maps to its own Catalogue string later; the log and UI name the reason,
/// never a bare "read-only". The reasons that found a directory carry it:
/// settings may still be readable there even when nothing can be written, while
/// an unknown own location has no directory at all — the payloads make the
/// difference unrepresentable rather than documented.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ReadOnlyReason {
    /// `current_exe()` failed (or reported a parentless path) — the app does
    /// not know where it is, so it cannot know where `data\` belongs.
    OwnLocationUnknown,
    /// The located `data\` could not be created.
    CannotCreate(PathBuf),
    /// `data\` exists but the write probe failed.
    NotWritable(PathBuf),
    /// The `--data-dir` location cannot be used (v0.2.0 §10): it could not be
    /// created, or its write probe failed, or the switch carried nothing that
    /// resolves — the last being the case with no directory to carry.
    ///
    /// One reason for all three because there is one thing to say about them
    /// and one thing to do about it, and because what it has to say is *which
    /// location* failed rather than how. It is emphatically not a step towards
    /// the default `data\`: this application never writes where it was not
    /// pointed.
    OverrideUnusable(Option<PathBuf>),
}

impl ReadOnlyReason {
    /// The Catalogue string naming this reason (spec §10.1 item 7): what the
    /// UI translates and fills into "Read-only: {reason}". Living beside the
    /// enum, a fifth reason cannot appear without naming its string.
    pub fn catalogue_msgid(&self) -> &'static str {
        match self {
            ReadOnlyReason::OwnLocationUnknown => msgids::READONLY_REASON_OWN_LOCATION_UNKNOWN,
            ReadOnlyReason::CannotCreate(_) => msgids::READONLY_REASON_CANNOT_CREATE,
            ReadOnlyReason::NotWritable(_) => msgids::READONLY_REASON_NOT_WRITABLE,
            ReadOnlyReason::OverrideUnusable(_) => msgids::READONLY_REASON_OVERRIDE_UNUSABLE,
        }
    }

    /// The directory this reason found, if it found one — how an override's
    /// failure keeps the location the default road's failure was carrying,
    /// while [`decide`] swaps the reason that names it.
    fn located(self) -> Option<PathBuf> {
        match self {
            ReadOnlyReason::CannotCreate(dir) | ReadOnlyReason::NotWritable(dir) => Some(dir),
            ReadOnlyReason::OverrideUnusable(dir) => dir,
            ReadOnlyReason::OwnLocationUnknown => None,
        }
    }
}

/// Where this Run's Data Directory came from — the locate step's answer, and
/// the one step `--data-dir` substitutes (v0.2.0 §10).
///
/// It is a separate question from [`DataDirState`], which is what became of it:
/// two Runs can end in the same Read-only Data for different reasons, and only
/// this says which of the two locations a Run was aimed at. That matters after
/// the decision as well as during it — the startup log line names an override
/// location, and an elevated relaunch has to be pointed at the same one or it
/// silently writes elsewhere.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Location {
    /// Beside the executable, the way every Run that was not pointed elsewhere
    /// finds it (ADR-0002).
    BesideExe(PathBuf),
    /// The process cannot say where it is, so it cannot say where `data\`
    /// belongs (spec §3).
    OwnLocationUnknown,
    /// `--data-dir`, resolved absolute once at startup — the single truth every
    /// downstream surface uses: the log line, the Read-only reason's record,
    /// and any self-relaunch's command line.
    Override(PathBuf),
    /// `--data-dir` given with nothing that resolves to an absolute path: a
    /// broken override, never an unknown argument, and never a fallback to the
    /// default `data\` (v0.2.0 §10).
    BrokenOverride,
}

impl Location {
    /// The path `--data-dir` resolved to, for the two surfaces that carry it
    /// beyond the decision: the startup log line and a self-relaunch's command
    /// line. `None` on a Run that was not pointed anywhere, and on one whose
    /// switch carried nothing to resolve — there is no path to name.
    pub fn override_path(&self) -> Option<&Path> {
        match self {
            Location::Override(dir) => Some(dir),
            Location::BesideExe(_) | Location::OwnLocationUnknown | Location::BrokenOverride => {
                None
            }
        }
    }

    /// Whether this Run was pointed by the switch at all — true for a broken
    /// override too, which has no path but still has to cross a process
    /// boundary as the switch it was.
    pub fn is_override(&self) -> bool {
        matches!(self, Location::Override(_) | Location::BrokenOverride)
    }
}

/// The Data Directory decision, made once at startup — a property of the run
/// (CONTEXT.md: Read-only Data). It governs the UI only: startup predicts,
/// Apply verifies by writing. Read-only Data never relocates the directory
/// and never prompts (ADR-0002) — hence no other constructor exists.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DataDirState {
    Writable(PathBuf),
    ReadOnly(ReadOnlyReason),
}

impl DataDirState {
    /// The directory this run's files live in, if it has one at all.
    ///
    /// Three of the four Read-only reasons can still name a directory, and one
    /// of those — a directory that exists but cannot be written — is exactly
    /// where a readable `settings.json` is most likely to be sitting; a Run
    /// pointed at an unwritable location by `--data-dir` is that same case, and
    /// obeys the language stored there for the same reason. An unknown own
    /// location has no answer, and neither has a broken override: there is no
    /// directory to have one.
    pub fn dir(&self) -> Option<&Path> {
        match self {
            DataDirState::Writable(dir) => Some(dir),
            DataDirState::ReadOnly(ReadOnlyReason::CannotCreate(dir)) => Some(dir),
            DataDirState::ReadOnly(ReadOnlyReason::NotWritable(dir)) => Some(dir),
            DataDirState::ReadOnly(ReadOnlyReason::OverrideUnusable(dir)) => dir.as_deref(),
            DataDirState::ReadOnly(ReadOnlyReason::OwnLocationUnknown) => None,
        }
    }

    /// Whether this run has a write path at all. Read-only Data closes every
    /// one of them — which includes the renames and rotations that are writes
    /// without being new content.
    pub fn is_writable(&self) -> bool {
        matches!(self, DataDirState::Writable(_))
    }

    /// The path-free fact the startup log line carries: the state, and when
    /// read-only the named reason — never the location (spec §14's PII
    /// prohibition on absolute paths in any record). The one path that line may
    /// carry rides beside this, out of [`Location`], not out of here.
    pub fn log_state(&self) -> logfmt::DataState {
        match self {
            DataDirState::Writable(_) => logfmt::DataState::Writable,
            DataDirState::ReadOnly(ReadOnlyReason::OwnLocationUnknown) => {
                logfmt::DataState::ReadOnlyOwnLocationUnknown
            }
            DataDirState::ReadOnly(ReadOnlyReason::CannotCreate(_)) => {
                logfmt::DataState::ReadOnlyCannotCreate
            }
            DataDirState::ReadOnly(ReadOnlyReason::NotWritable(_)) => {
                logfmt::DataState::ReadOnlyNotWritable
            }
            DataDirState::ReadOnly(ReadOnlyReason::OverrideUnusable(_)) => {
                logfmt::DataState::ReadOnlyOverrideUnusable
            }
        }
    }
}

/// The locate rule (spec §3): reported executable path → resolve reparse
/// points → strip the verbatim prefix → parent → append `data`. Resolution
/// goes through `fs::canonicalize` because `current_exe()` reports a launcher
/// junction, not its target (measured under winget — the naive rule would put
/// `data\` in the shared `Links\` directory). Resolution failure falls back
/// to the unresolved path; a parentless path has no answer.
pub fn locate(exe_path: &Path) -> Option<PathBuf> {
    let resolved = fs::canonicalize(exe_path).unwrap_or_else(|_| exe_path.to_path_buf());
    Some(strip_verbatim_prefix(&resolved).parent()?.join("data"))
}

/// The reason selection on a locate answer — the whole Data Directory decision
/// minus the two calls (`current_exe()`, `current_dir()`) a test cannot make
/// fail. Those stay at the edge, in `main`, which hands the answer down through
/// [`startup::decide`](crate::startup::decide); there is deliberately no
/// convenience that bundles them, because a second route to this decision would
/// bypass the writability rule built on top of it (ADR-0010).
///
/// **`--data-dir` substitutes the locate step and nothing else** (v0.2.0 §10):
/// an override runs the same create-and-probe as the default `data\`, so a
/// missing directory is created there exactly as it would be here. Only the
/// *reason* a failure earns differs, and it differs in the one way that helps
/// — it names the switch, so the user hears which of the two locations failed.
/// There is no path from here back to the default: an application that fell
/// back would be writing where it was not pointed, which is the hazard this
/// switch exists inside, not beside.
pub fn decide(location: Location) -> DataDirState {
    match location {
        Location::BesideExe(dir) => establish(dir),
        Location::OwnLocationUnknown => DataDirState::ReadOnly(ReadOnlyReason::OwnLocationUnknown),
        Location::Override(dir) => match establish(dir) {
            writable @ DataDirState::Writable(_) => writable,
            DataDirState::ReadOnly(reason) => {
                DataDirState::ReadOnly(ReadOnlyReason::OverrideUnusable(reason.located()))
            }
        },
        Location::BrokenOverride => DataDirState::ReadOnly(ReadOnlyReason::OverrideUnusable(None)),
    }
}

/// The locate step under `--data-dir` (v0.2.0 §10): the switch's value cleaned
/// of the artifacts Windows' own parsing leaves on it, then made absolute
/// against `cwd`.
///
/// **Made absolute, never canonicalised.** `fs::canonicalize` answers with a
/// `\\?\` path, and this one has to survive a command line — the elevated
/// instance is handed it verbatim. It also touches the filesystem, and a
/// directory that does not exist yet is a perfectly good target: creating it is
/// the very next step.
///
/// **Relative to the current directory**, which is what every verifiable
/// precedent does and what a shell user means. The one relative shape this
/// cannot answer for is the drive-relative `C:foo`, which names a current
/// directory *per drive* that only the OS knows: rather than guess at it, that
/// is a broken override — the whole point of the switch is that the application
/// writes where it was pointed, and a guess is not a pointing.
pub fn locate_override(value: &OsStr, cwd: &Path) -> Location {
    let cleaned = trim_override_value(value);
    if cleaned.is_empty() {
        return Location::BrokenOverride;
    }
    let given = Path::new(&cleaned);
    let absolute = if given.is_absolute() {
        given.to_path_buf()
    } else {
        cwd.join(given)
    };
    match absolute.is_absolute() {
        true => Location::Override(absolute),
        false => Location::BrokenOverride,
    }
}

/// Strips the two artifacts Windows' command-line parsing leaves on a quoted
/// path (v0.2.0 §10), in the order they are left.
///
/// A trailing `"` is the backslash-before-quote rule showing through:
/// `--data-dir "C:\x\"` reaches the process as `C:\x"`, because the backslash
/// escaped the quote that was meant to close it. Trailing separators are the
/// other half of the same reflex — `"C:\x\\"` arrives as `C:\x\` — and they
/// name the same directory as no separator at all, but not the same string, and
/// this string is what the log and the elevated relaunch both carry.
///
/// Never past the root: `C:\` is a directory, where `C:` is a drive-relative
/// path that names a different one.
fn trim_override_value(value: &OsStr) -> OsString {
    const QUOTE: u16 = b'"' as u16;
    const SEPARATORS: [u16; 2] = [b'\\' as u16, b'/' as u16];

    let mut units: Vec<u16> = value.encode_wide().collect();
    while units.last() == Some(&QUOTE) {
        units.pop();
    }
    while units.last().is_some_and(|last| SEPARATORS.contains(last)) {
        let shorter = OsString::from_wide(&units[..units.len() - 1]);
        // A path with no parent is a root, and a root's separator is part of it.
        if shorter.is_empty() || Path::new(&shorter).parent().is_none() {
            break;
        }
        units.pop();
    }
    OsString::from_wide(&units)
}

/// Create-and-probe half of the startup sequence: `create_dir_all`, then a
/// pid-unique transient probe file. The mode this returns is decided exactly
/// once per run — callers hold the result, they never re-establish. Public so
/// tests aim the decision at temp directories (the same seam-as-parameter
/// pattern as `registry::ScopeKey::at`).
///
/// It runs unchanged over an override (v0.2.0 §10): [`decide`] renames what it
/// answers, never what it does.
pub fn establish(dir: PathBuf) -> DataDirState {
    if fs::create_dir_all(&dir).is_err() {
        return DataDirState::ReadOnly(ReadOnlyReason::CannotCreate(dir));
    }
    if !probe_write(&dir) {
        return DataDirState::ReadOnly(ReadOnlyReason::NotWritable(dir));
    }
    DataDirState::Writable(dir)
}

/// The atomic-replace write every later file consumer (settings, Snapshots)
/// goes through (spec §3): content lands in a pid-unique `*.tmp` beside the
/// target, which then replaces it in one `MoveFileExW(MOVEFILE_REPLACE_EXISTING)`
/// — a reader never sees a half-written file. Pid-unique because two
/// instances are a designed state (no single-instance lock exists); same
/// directory because the rename is only atomic within a volume.
pub fn write_replace(target: &Path, contents: &[u8]) -> io::Result<()> {
    let file_name = target
        .file_name()
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "target has no file name"))?;
    let mut temp_name = file_name.to_os_string();
    temp_name.push(format!(".{}.tmp", std::process::id()));
    let temp = target.with_file_name(temp_name);
    fs::write(&temp, contents)?;
    rename_replacing(&temp, target).inspect_err(|_| {
        let _ = fs::remove_file(&temp);
    })
}

/// The rename half of [`write_replace`], for the callers that already have the
/// file they want to move — setting `settings.json` aside as `settings.json.bad`
/// is a rename, not a copy, so the bad file is never read twice and never half
/// present. Replacing is what keeps that copy single (spec §13).
pub fn rename_replacing(from: &Path, to: &Path) -> io::Result<()> {
    let from_wide = nul_terminated_wide(from);
    let to_wide = nul_terminated_wide(to);
    // SAFETY: both pointers are NUL-terminated UTF-16 buffers that outlive the call.
    let moved = unsafe {
        MoveFileExW(
            from_wide.as_ptr(),
            to_wide.as_ptr(),
            MOVEFILE_REPLACE_EXISTING,
        )
    };
    if moved == 0 {
        Err(io::Error::last_os_error())
    } else {
        Ok(())
    }
}

fn nul_terminated_wide(path: &Path) -> Vec<u16> {
    path.as_os_str()
        .encode_wide()
        .chain(std::iter::once(0))
        .collect()
}

/// Writes and deletes a pid-unique probe file (TC-file-structure's "transient
/// pid-unique write probe"). Pid-unique because two instances are a designed
/// state — an elevated relaunch may probe the same directory concurrently.
fn probe_write(dir: &Path) -> bool {
    let probe = dir.join(format!("probe-{}.tmp", std::process::id()));
    let created = fs::write(&probe, b"pathmaster write probe").is_ok();
    let _ = fs::remove_file(&probe);
    created
}

/// Rewrites `fs::canonicalize`'s verbatim output into the plain form the rest
/// of the app (and the user's eyes) work with: `\\?\C:\…` → `C:\…`,
/// `\\?\UNC\server\share\…` → `\\server\share\…`. Any other shape is
/// returned untouched. Public so the mangling steps are testable without a
/// filesystem to canonicalize against.
pub fn strip_verbatim_prefix(path: &Path) -> PathBuf {
    let mut components = path.components();
    let plain_root = match components.next() {
        Some(Component::Prefix(prefix)) => match plain_form(prefix) {
            Some(root) => root,
            None => return path.to_path_buf(),
        },
        _ => return path.to_path_buf(),
    };
    let mut out = PathBuf::from(plain_root);
    for component in components {
        // The RootDir after the prefix is already part of the rebuilt root.
        if !matches!(component, Component::RootDir) {
            out.push(component.as_os_str());
        }
    }
    out
}

fn plain_form(prefix: PrefixComponent<'_>) -> Option<OsString> {
    match prefix.kind() {
        Prefix::VerbatimDisk(letter) => Some(OsString::from(format!(r"{}:\", letter as char))),
        Prefix::VerbatimUNC(server, share) => {
            let mut root = OsString::from(r"\\");
            root.push(server);
            root.push(r"\");
            root.push(share);
            root.push(r"\");
            Some(root)
        }
        _ => None,
    }
}
