//! Elevation: detection, and the whole-app relaunch that is the only way into
//! it (spec §9, ADR-0005).
//!
//! The relaunch carries **one argument across the process boundary — the
//! active tab, nothing else** (ticket 12 D5). Sessions are dead at the
//! boundary and stay dead; [`StartTab`] is both the writer and the reader of
//! that argument, so the two instances cannot drift apart about its spelling.

use std::mem;
use std::os::windows::ffi::OsStrExt;

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

/// The tab the user left, named for the relaunch's one argument (ticket 12
/// D5). It is a *tab* and not a Scope because the Backups tab is one of the
/// places the user can be — `CONTEXT.md` keeps "Tab" off **Scope** for
/// exactly this reason.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StartTab {
    User,
    System,
    Backups,
}

impl StartTab {
    /// Every tab, for the callers that search rather than match — the reader
    /// below, and the window's inverse lookup.
    pub const ALL: [StartTab; 3] = [StartTab::User, StartTab::System, StartTab::Backups];

    /// The value the spawner writes after `--tab`. [`from_args`](Self::from_args)
    /// reads by searching this same function over [`ALL`](Self::ALL), so the
    /// pair round-trips by construction — there is no second spelling to
    /// drift.
    pub fn argument(self) -> &'static str {
        match self {
            StartTab::User => "user",
            StartTab::System => "system",
            StartTab::Backups => "backups",
        }
    }

    /// The tab a command line asks to open on: the value after the first
    /// `--tab`, if it is one this application's own spawner writes.
    ///
    /// Anything else — no `--tab`, a bare one, a value nothing writes — is
    /// `None`, never a guess: the caller's default (the User tab, the same
    /// one a plain launch opens on) is a better answer than a misreading.
    pub fn from_args(args: impl IntoIterator<Item = String>) -> Option<StartTab> {
        let mut args = args.into_iter();
        args.find(|arg| arg == "--tab")?;
        let value = args.next()?;
        Self::ALL.into_iter().find(|tab| tab.argument() == value)
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
/// current exe with `--tab <active>`, the one argument that crosses the
/// boundary (ADR-0005). On `Ok` the elevated instance is up and the caller's
/// contract is to exit; on `Err` nothing was spawned and this instance keeps
/// running.
///
/// The exe path is asked for here, not taken from the caller: the relaunch
/// must aim at the binary this process is actually running, the same answer
/// `main` located the Data Directory from.
pub fn relaunch_elevated(tab: StartTab) -> Result<(), RelaunchFailure> {
    let exe = std::env::current_exe().map_err(|error| RelaunchFailure::Failed {
        os_error: error.raw_os_error(),
    })?;

    // UTF-16, NUL-terminated, and alive until the call returns.
    let verb: Vec<u16> = "runas".encode_utf16().chain([0]).collect();
    let file: Vec<u16> = exe.as_os_str().encode_wide().chain([0]).collect();
    let parameters: Vec<u16> = format!("--tab {}", tab.argument())
        .encode_utf16()
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
