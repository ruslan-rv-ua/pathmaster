//! The log writer (spec §14): `pathmaster.log` in the Data Directory, one
//! line per record, rotation only at open. No logging failure touches the
//! app — every record is an independent attempt, failures are dropped and
//! counted, and an unopenable log at startup is a run without a log, never
//! Read-only Data.

use std::fs::{self, File, OpenOptions};
use std::io::Write as _;
use std::mem;
use std::os::windows::fs::OpenOptionsExt as _;
use std::path::{Path, PathBuf};

use pathmaster_core::logfmt::{line, Record, Timestamp};
use windows_sys::Win32::Foundation::{FILETIME, SYSTEMTIME};
use windows_sys::Win32::Storage::FileSystem::{FILE_SHARE_READ, FILE_SHARE_WRITE};
use windows_sys::Win32::System::SystemInformation::{GetLocalTime, GetSystemTime};
use windows_sys::Win32::System::Time::SystemTimeToFileTime;

pub const LOG_FILE_NAME: &str = "pathmaster.log";
pub const OLD_FILE_NAME: &str = "pathmaster.log.old";

/// Rotation threshold — *over* 1 MB at open (spec §14). At minimal-logging
/// rates this is years of history.
const ROTATE_OVER_BYTES: u64 = 1_048_576;

/// The run's logger. Opened once at startup against the Data Directory;
/// holds no file handle — each record opens, appends one line and closes, so
/// a failure never latches and another instance (or the panic hook) can
/// always append to the same file.
#[derive(Debug)]
pub struct Logger {
    target: Option<PathBuf>,
    lost: u64,
}

impl Logger {
    /// Rotation, then the open probe. Over 1 MB → rename to
    /// `pathmaster.log.old`, overwriting the single previous generation; a
    /// failed rename (another instance holds the file) carries on appending.
    /// An unopenable log is a run without a log — never an error.
    pub fn open(data_dir: &Path) -> Logger {
        let target = data_dir.join(LOG_FILE_NAME);
        let oversized = fs::metadata(&target).is_ok_and(|m| m.len() > ROTATE_OVER_BYTES);
        if oversized {
            let _ = fs::rename(&target, data_dir.join(OLD_FILE_NAME));
        }
        match append_handle(&target) {
            Ok(_) => Logger {
                target: Some(target),
                lost: 0,
            },
            Err(_) => Logger::disabled(),
        }
    }

    /// A run without a log — for Read-only Data, where the log has no home.
    pub fn disabled() -> Logger {
        Logger {
            target: None,
            lost: 0,
        }
    }

    /// Where records go, if anywhere — the panic hook installs against this
    /// so it can append past the logger.
    pub fn path(&self) -> Option<&Path> {
        self.target.as_deref()
    }

    /// One independent attempt: stamp, format, open, append, close. A failed
    /// write is silently dropped and counted; the first success after losses
    /// prepends one `WARN log: N records were lost`. Nothing here can fail
    /// outward.
    pub fn log(&mut self, record: &Record) {
        let Some(target) = &self.target else { return };
        let stamp = now();
        let mut text = String::new();
        if self.lost > 0 {
            text.push_str(&line(&stamp, &Record::records_lost(self.lost)));
        }
        text.push_str(&line(&stamp, record));
        let attempt = append_handle(target).and_then(|mut file| file.write_all(text.as_bytes()));
        match attempt {
            Ok(()) => self.lost = 0,
            Err(_) => self.lost += 1,
        }
    }
}

/// Append-or-create with share read/write (spec §14) — concurrent readers, a
/// second instance and the panic hook are never locked out. Deliberately no
/// delete-sharing: while a write is in flight, another instance's rotation
/// rename fails and that instance carries on appending, per the spec's
/// failed-rename rule. Shared with the panic hook, which appends past the
/// `Logger` but through the same file semantics.
pub(crate) fn append_handle(target: &Path) -> std::io::Result<File> {
    OpenOptions::new()
        .append(true)
        .create(true)
        .share_mode(FILE_SHARE_READ | FILE_SHARE_WRITE)
        .open(target)
}

/// The local wall clock with its real UTC offset, read from Win32. The offset
/// is the measured local−UTC difference rounded to the minute — immune to the
/// standard/daylight bias-selection mistakes of `GetTimeZoneInformation`.
pub fn now() -> Timestamp {
    // SAFETY: SYSTEMTIME is a plain all-u16 struct; both calls only write it.
    let (local, utc) = unsafe {
        let mut local: SYSTEMTIME = mem::zeroed();
        let mut utc: SYSTEMTIME = mem::zeroed();
        GetSystemTime(&mut utc);
        GetLocalTime(&mut local);
        (local, utc)
    };
    Timestamp {
        year: local.wYear,
        month: local.wMonth as u8,
        day: local.wDay as u8,
        hour: local.wHour as u8,
        minute: local.wMinute as u8,
        second: local.wSecond as u8,
        offset_minutes: offset_minutes(&local, &utc),
    }
}

fn offset_minutes(local: &SYSTEMTIME, utc: &SYSTEMTIME) -> i32 {
    const MINUTE_100NS: i64 = 60 * 10_000_000;
    match (filetime_100ns(local), filetime_100ns(utc)) {
        // The two clock reads may straddle a second boundary; rounding to the
        // nearest minute absorbs it (real offsets are quarter-hour granular).
        (Some(l), Some(u)) => ((l - u + MINUTE_100NS / 2).div_euclid(MINUTE_100NS)) as i32,
        _ => 0,
    }
}

fn filetime_100ns(st: &SYSTEMTIME) -> Option<i64> {
    let mut ft = FILETIME {
        dwLowDateTime: 0,
        dwHighDateTime: 0,
    };
    // SAFETY: plain struct-in/struct-out conversion call.
    let converted = unsafe { SystemTimeToFileTime(st, &mut ft) };
    (converted != 0).then_some(((ft.dwHighDateTime as i64) << 32) | ft.dwLowDateTime as i64)
}
