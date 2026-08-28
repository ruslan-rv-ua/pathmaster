//! Handing something to whatever the user has set to open it.
//!
//! Two callers, one spelling of `ShellExecuteW`: Tools → Open Backups Folder
//! shows a directory (spec §15), and F1 shows the User Guide — a file, or on
//! the rung below it a URL (v0.2.0 §9). All three are the same call, and the
//! shell is what decides which program answers.
//!
//! This is emphatically **not** a file dialog and never becomes one: nothing
//! here asks the user for anything.

use std::ffi::OsStr;
use std::os::windows::ffi::OsStrExt;

use windows_sys::Win32::UI::Shell::ShellExecuteW;
use windows_sys::Win32::UI::WindowsAndMessaging::SW_SHOWNORMAL;

/// The shell verb: open the thing, whatever the user has set to open it. Not
/// `explore`, which names one program — this is the shell's own answer.
const OPEN: &str = "open";

/// `ShellExecuteW`'s success line: an `HINSTANCE` **above 32** is a launch,
/// and every value at or below it is one of the documented error codes
/// (`SE_ERR_NOASSOC` and its siblings). The API is old enough to answer in a
/// handle-shaped integer that is not a handle.
const SUCCESS_FLOOR: isize = 32;

/// Opens `target` — a directory, a file, or a URL — through the shell.
///
/// Answers **whether anything opened**, which the two callers use differently
/// and deliberately: the Backups folder is silence either way (its only
/// failing run is one whose Data Directory does not exist either), while the
/// User Guide's bottom rung earns a log line, because it is the one rung the
/// user cannot see they are on (v0.2.0 §9).
///
/// No owning window is passed, so the shell's own error UI has none either.
/// That is the same choice `elevation` makes for the opposite reason: there,
/// `ShellExecuteEx`'s error dialog is wanted; here nothing is expected to go
/// wrong that the user could act on.
pub fn open(target: &OsStr) -> bool {
    let target = nul_terminated_wide(target);
    let verb: Vec<u16> = OPEN.encode_utf16().chain(std::iter::once(0)).collect();
    // SAFETY: both pointers are NUL-terminated UTF-16 buffers that outlive the
    // call, and the three nulls are the documented "no parameters, no working
    // directory, no window to own the error" arguments.
    let result = unsafe {
        ShellExecuteW(
            std::ptr::null_mut(),
            verb.as_ptr(),
            target.as_ptr(),
            std::ptr::null(),
            std::ptr::null(),
            SW_SHOWNORMAL,
        )
    };
    result as isize > SUCCESS_FLOOR
}

fn nul_terminated_wide(text: &OsStr) -> Vec<u16> {
    text.encode_wide().chain(std::iter::once(0)).collect()
}
