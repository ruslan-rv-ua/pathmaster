//! Elevation detection (spec §9, ADR-0005). Detection only — the "Restart as
//! Administrator" relaunch is ticket 17's.

use std::mem;

use windows_sys::Win32::Foundation::{CloseHandle, HANDLE};
use windows_sys::Win32::Security::{
    GetTokenInformation, TokenElevation, TOKEN_ELEVATION, TOKEN_QUERY,
};
use windows_sys::Win32::System::Threading::{GetCurrentProcess, OpenProcessToken};

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
