//! Elevation: detection, and the whole-app relaunch that is the only way into
//! it (spec §9, ADR-0005).
//!
//! The relaunch carries **the active tab and this Run's Data Directory
//! override, and nothing else** (ticket 12 D5; v0.2.0 §10). Sessions are dead
//! at the boundary and stay dead. The tab is where the user was; the override
//! is where this Run writes, and it has to cross or the elevated instance
//! silently writes somewhere else — which is the general rule, not a fact about
//! elevation: **any future self-relaunch carries the override**.
//!
//! What crosses is built by [`CommandLine::relaunch`] out of *parsed state*,
//! never out of this process's own command-line tail — see there for why the
//! resolved absolute path is the only spelling that can make the trip.

use std::mem;
use std::os::windows::ffi::OsStrExt;

use crate::args::{CommandLine, StartTab};
use crate::datadir::Location;

use windows_sys::Win32::Foundation::{CloseHandle, GetLastError, ERROR_CANCELLED, HANDLE};
use windows_sys::Win32::Security::{
    GetTokenInformation, TokenElevation, TOKEN_ELEVATION, TOKEN_QUERY,
};
use windows_sys::Win32::System::Threading::{GetCurrentProcess, OpenProcessToken};
use windows_sys::Win32::UI::Shell::{ShellExecuteExW, SEE_MASK_NOASYNC, SHELLEXECUTEINFOW};
use windows_sys::Win32::UI::WindowsAndMessaging::SW_SHOWNORMAL;

/// Whether this process runs elevated, via `GetTokenInformation(TokenElevation)`
/// — never `TokenElevationType`, which misreads built-in-admin and UAC-off
/// machines (spec §9). Called once at startup; the answer is a property of
/// the process and cannot change while it runs. Any query failure reads as
/// not elevated — the degraded answer, never the privileged one.
pub fn is_elevated() -> bool {
    let mut token: HANDLE = std::ptr::null_mut();
    // SAFETY: plain handle-out Win32 calls; the token handle is closed on
    // every path that opened it.
    unsafe {
        if OpenProcessToken(GetCurrentProcess(), TOKEN_QUERY, &mut token) == 0 {
            return false;
        }
        let mut elevation = TOKEN_ELEVATION { TokenIsElevated: 0 };
        let mut returned_len = 0u32;
        let queried = GetTokenInformation(
            token,
            TokenElevation,
            &mut elevation as *mut TOKEN_ELEVATION as *mut _,
            mem::size_of::<TOKEN_ELEVATION>() as u32,
            &mut returned_len,
        );
        CloseHandle(token);
        queried != 0 && elevation.TokenIsElevated != 0
    }
}

/// Why the elevated instance did not start. The first variant is the one the
/// spec names (§9): the user answered the UAC prompt with No, and silence
/// after a security prompt is treated as a defect — the caller owes a dialog.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RelaunchFailure {
    /// `ERROR_CANCELLED` (1223): the UAC prompt was declined. The application
    /// carries on unelevated, fully functional.
    Declined,
    /// Anything else, with the raw error for the caller's log line. When the
    /// `ShellExecuteEx` call itself ran, it has already shown its own error
    /// UI — `SEE_MASK_FLAG_NO_UI` is deliberately not set — but the one path
    /// that fails before it, a process that cannot name its own executable,
    /// shows nothing: the log line is that path's only witness, which is why
    /// the code travels rather than being dropped here.
    Failed { os_error: Option<i32> },
}

/// Relaunches this executable elevated — `ShellExecuteEx("runas")` on the
/// current exe with the line [`CommandLine::relaunch`] builds (ADR-0005;
/// v0.2.0 §10). On `Ok` the elevated instance is up and the caller's contract
/// is to exit; on `Err` nothing was spawned and this instance keeps running.
///
/// The exe path is asked for here, not taken from the caller: the relaunch
/// must aim at the binary this process is actually running, the same answer
/// `main` located the Data Directory from. `location` is the opposite — it is
/// this Run's own decision and cannot be re-derived here, because re-deriving
/// it is exactly what would lose a relative path's meaning.
pub fn relaunch_elevated(tab: StartTab, location: &Location) -> Result<(), RelaunchFailure> {
    let exe = std::env::current_exe().map_err(|error| RelaunchFailure::Failed {
        os_error: error.raw_os_error(),
    })?;

    // UTF-16, NUL-terminated, and alive until the call returns.
    let verb: Vec<u16> = "runas".encode_utf16().chain([0]).collect();
    let file: Vec<u16> = exe.as_os_str().encode_wide().chain([0]).collect();
    let parameters: Vec<u16> = CommandLine::relaunch(tab, location)
        .into_os_string()
        .encode_wide()
        .chain([0])
        .collect();

    // SAFETY: the struct is fully initialised below, every pointer it carries
    // outlives the call, and `SEE_MASK_NOASYNC` makes the spawn synchronous —
    // required, because the caller exits as soon as this returns Ok.
    unsafe {
        let mut info: SHELLEXECUTEINFOW = mem::zeroed();
        info.cbSize = mem::size_of::<SHELLEXECUTEINFOW>() as u32;
        info.fMask = SEE_MASK_NOASYNC;
        info.lpVerb = verb.as_ptr();
        info.lpFile = file.as_ptr();
        info.lpParameters = parameters.as_ptr();
        info.nShow = SW_SHOWNORMAL;
        if ShellExecuteExW(&mut info) != 0 {
            return Ok(());
        }
        match GetLastError() {
            ERROR_CANCELLED => Err(RelaunchFailure::Declined),
            other => Err(RelaunchFailure::Failed {
                os_error: Some(other as i32),
            }),
        }
    }
}
