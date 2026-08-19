//! The Data Directory: located from the executable, never relocated
//! (spec §3, ADR-0002).

use std::ffi::OsString;
use std::fs;
use std::io;
use std::os::windows::ffi::OsStrExt;
use std::path::{Component, Path, PathBuf, Prefix, PrefixComponent};

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

/// The Data Directory decision, made once at startup — a property of the run
/// (CONTEXT.md: Read-only Data). It governs the UI only: startup predicts,
/// Apply verifies by writing. Read-only Data never relocates the directory
/// and never prompts (ADR-0002) — hence no other constructor exists.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DataDirState {
    Writable(PathBuf),
    ReadOnly(ReadOnlyReason),
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

/// The whole Data Directory decision, made from the running process (spec
/// §3's startup tree — the log and settings steps that follow it belong to
/// later tickets): locate from `current_exe()`, then decide.
pub fn startup() -> DataDirState {
    decide(std::env::current_exe().ok().as_deref().and_then(locate))
}

/// The reason selection on a locate answer — `startup()` minus the one call
/// (`current_exe()`) a test cannot make fail; public as that test seam.
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
    move_replacing(&temp, target).inspect_err(|_| {
        let _ = fs::remove_file(&temp);
    })
}

fn move_replacing(from: &Path, to: &Path) -> io::Result<()> {
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
