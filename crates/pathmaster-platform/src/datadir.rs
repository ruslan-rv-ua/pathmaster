//! The Data Directory: located from the executable, never relocated
//! (spec §3, ADR-0002).

use std::ffi::OsString;
use std::fs;
use std::io;
use std::os::windows::ffi::OsStrExt;
use std::path::{Component, Path, PathBuf, Prefix, PrefixComponent};

use pathmaster_core::logfmt;
use pathmaster_core::msgids;
use windows_sys::Win32::Storage::FileSystem::{MoveFileExW, MOVEFILE_REPLACE_EXISTING};

/// Why a run is in Read-only Data — exactly these three (spec §3). Each maps
/// to its own Catalogue string later; the log and UI name the reason, never a
/// bare "read-only". The two reasons that found a directory carry it: settings
/// may still be readable there even when nothing can be written, while an
/// unknown own location has no directory at all — the payloads make the
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
}

impl ReadOnlyReason {
    /// The Catalogue string naming this reason (spec §10.1 item 7): what the
    /// UI translates and fills into "Read-only: {reason}". Living beside the
    /// enum, a fourth reason cannot appear without naming its string.
    pub fn catalogue_msgid(&self) -> &'static str {
        match self {
            ReadOnlyReason::OwnLocationUnknown => msgids::READONLY_REASON_OWN_LOCATION_UNKNOWN,
            ReadOnlyReason::CannotCreate(_) => msgids::READONLY_REASON_CANNOT_CREATE,
            ReadOnlyReason::NotWritable(_) => msgids::READONLY_REASON_NOT_WRITABLE,
        }
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
    /// Two of the three Read-only reasons still name a directory, and one of
    /// those — a directory that exists but cannot be written — is exactly where
    /// a readable `settings.json` is most likely to be sitting. Only an unknown
    /// own location has no answer, because there is no directory to have one.
    pub fn dir(&self) -> Option<&Path> {
        match self {
            DataDirState::Writable(dir) => Some(dir),
            DataDirState::ReadOnly(ReadOnlyReason::CannotCreate(dir)) => Some(dir),
            DataDirState::ReadOnly(ReadOnlyReason::NotWritable(dir)) => Some(dir),
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
    /// prohibition on absolute paths in any record).
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
/// minus the one call (`current_exe()`) a test cannot make fail. That call
/// stays at the edge, in `main`, which hands the answer down through
/// [`startup::decide`](crate::startup::decide); there is deliberately no
/// convenience that bundles the two, because a second route to this decision
/// would bypass the writability rule built on top of it (ADR-0010).
/// A run with no located directory cannot know where it is: Read-only Data,
/// own location unknown.
pub fn decide(located: Option<PathBuf>) -> DataDirState {
    match located {
        Some(dir) => establish(dir),
        None => DataDirState::ReadOnly(ReadOnlyReason::OwnLocationUnknown),
    }
}

/// Create-and-probe half of the startup sequence: `create_dir_all`, then a
/// pid-unique transient probe file. The mode this returns is decided exactly
/// once per run — callers hold the result, they never re-establish. Public so
/// tests aim the decision at temp directories (the same seam-as-parameter
/// pattern as `registry::ScopeKey::at`).
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
