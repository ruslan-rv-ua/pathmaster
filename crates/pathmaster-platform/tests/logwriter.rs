//! The log writer at the crate boundary (spec §14, ticket impl-05), against
//! real files: rotation only at open, every record an independent attempt,
//! and no logging failure that touches the app.

#![cfg(windows)]

use std::fs;
use std::os::windows::fs::OpenOptionsExt;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

use pathmaster_core::logfmt::Record;
use pathmaster_platform::logwriter::{now, Logger, LOG_FILE_NAME, OLD_FILE_NAME};

fn read_log(dir: &Path) -> String {
    fs::read_to_string(dir.join(LOG_FILE_NAME)).expect("log file should exist")
}

#[test]
fn a_logged_record_lands_as_one_line_in_pathmaster_log() {
    let dir = tempfile::tempdir().unwrap();
    let mut logger = Logger::open(dir.path());
    logger.log(&Record::shutdown_clean());
    let text = read_log(dir.path());
    assert_eq!(text.lines().count(), 1, "{text:?}");
    assert!(text.ends_with("INFO  shutdown: clean\n"), "{text:?}");
    // The line starts with an RFC 3339 local timestamp: `2026-08-19T15:36:31+03:00 `.
    assert_eq!(text.as_bytes()[10], b'T', "{text:?}");
    assert_eq!(text.as_bytes()[25], b' ', "{text:?}");
}

/// Independent oracle for `now()`: rebuild the Unix time from the reported
/// calendar fields and offset (civil-days arithmetic, nothing shared with the
/// implementation) and compare against `SystemTime::now()`.
#[test]
fn now_reports_the_local_wall_clock_with_its_real_offset() {
    let ts = now();
    let days = days_from_civil(i64::from(ts.year), i64::from(ts.month), i64::from(ts.day));
    let local_secs = days * 86_400
        + i64::from(ts.hour) * 3_600
        + i64::from(ts.minute) * 60
        + i64::from(ts.second);
    let unix_secs = local_secs - i64::from(ts.offset_minutes) * 60;
    let std_now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64;
    assert!(
        (unix_secs - std_now).abs() <= 5,
        "now() = {unix_secs}, SystemTime = {std_now}",
    );
    assert_eq!(
        ts.offset_minutes % 15,
        0,
        "offsets are quarter-hour granular"
    );
}

// Howard Hinnant's days_from_civil — an independent recomputation for the
// oracle above, not shared with the implementation (which reads Win32 time).
fn days_from_civil(y: i64, m: i64, d: i64) -> i64 {
    let y = if m <= 2 { y - 1 } else { y };
    let era = if y >= 0 { y } else { y - 399 } / 400;
    let yoe = y - era * 400;
    let doy = (153 * (if m > 2 { m - 3 } else { m + 9 }) + 2) / 5 + d - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    era * 146_097 + doe - 719_468
}

#[test]
fn an_oversized_log_rotates_to_old_at_open_overwriting_the_previous_old() {
    let dir = tempfile::tempdir().unwrap();
    let log = dir.path().join(LOG_FILE_NAME);
    let old = dir.path().join(OLD_FILE_NAME);
    fs::write(&log, "fresh generation ".repeat(70_000)).unwrap(); // > 1 MB
    fs::write(&old, "stale generation").unwrap();
    let mut logger = Logger::open(dir.path());
    assert!(
        fs::read_to_string(&old)
            .unwrap()
            .starts_with("fresh generation"),
        "the single .old generation is overwritten",
    );
    logger.log(&Record::shutdown_clean());
    let text = read_log(dir.path());
    assert_eq!(
        text.lines().count(),
        1,
        "rotation left a fresh log: {text:?}"
    );
}

#[test]
fn a_log_at_exactly_one_megabyte_does_not_rotate() {
    let dir = tempfile::tempdir().unwrap();
    let log = dir.path().join(LOG_FILE_NAME);
    fs::write(&log, "x".repeat(1_048_576)).unwrap();
    let _logger = Logger::open(dir.path());
    assert!(!dir.path().join(OLD_FILE_NAME).exists());
    assert_eq!(fs::metadata(&log).unwrap().len(), 1_048_576);
}

#[test]
fn a_failed_rotation_rename_carries_on_appending() {
    let dir = tempfile::tempdir().unwrap();
    let log = dir.path().join(LOG_FILE_NAME);
    fs::write(&log, "y".repeat(1_100_000)).unwrap();
    // Another instance holds the log open without FILE_SHARE_DELETE — the
    // rename fails, and the logger appends to the oversized file instead.
    let _holder = fs::OpenOptions::new()
        .read(true)
        .share_mode(0x1 | 0x2) // FILE_SHARE_READ | FILE_SHARE_WRITE
        .open(&log)
        .unwrap();
    let mut logger = Logger::open(dir.path());
    logger.log(&Record::shutdown_clean());
    let text = read_log(dir.path());
    assert!(text.len() > 1_100_000, "still the same generation");
    assert!(
        text.ends_with("INFO  shutdown: clean\n"),
        "appended past the failed rename"
    );
    assert!(!dir.path().join(OLD_FILE_NAME).exists());
}

#[test]
fn an_unopenable_log_at_startup_is_a_run_without_a_log() {
    let dir = tempfile::tempdir().unwrap();
    // A directory squatting on the log's name makes it unopenable.
    fs::create_dir(dir.path().join(LOG_FILE_NAME)).unwrap();
    let mut logger = Logger::open(dir.path());
    assert_eq!(logger.path(), None);
    // Logging into the void must be silent — no panic, no error surfaced.
    logger.log(&Record::shutdown_clean());
    logger.log(&Record::records_lost(1));
}

// The lint guards Unix (where clearing readonly means world-writable); this
// test is Windows-only and toggles the actual FILE_ATTRIBUTE_READONLY bit.
#[allow(clippy::permissions_set_readonly_false)]
#[test]
fn failed_writes_are_dropped_counted_and_announced_once_on_recovery() {
    let dir = tempfile::tempdir().unwrap();
    let log = dir.path().join(LOG_FILE_NAME);
    let mut logger = Logger::open(dir.path());
    logger.log(&Record::startup(
        "0.1.0",
        false,
        pathmaster_core::logfmt::DataState::Writable,
        "en",
        None,
    ));

    let mut readonly = fs::metadata(&log).unwrap().permissions();
    readonly.set_readonly(true);
    fs::set_permissions(&log, readonly.clone()).unwrap();
    for _ in 0..3 {
        logger.log(&Record::shutdown_clean()); // dropped silently
    }
    readonly.set_readonly(false);
    fs::set_permissions(&log, readonly).unwrap();

    logger.log(&Record::shutdown_clean());
    let text = read_log(dir.path());
    let lines: Vec<&str> = text.lines().collect();
    assert_eq!(lines.len(), 3, "{text:?}");
    assert!(lines[0].contains("INFO  startup:"), "{text:?}");
    assert!(
        lines[1].ends_with("WARN  log: 3 records were lost"),
        "{text:?}"
    );
    assert!(lines[2].ends_with("INFO  shutdown: clean"), "{text:?}");
}

#[test]
fn two_instances_append_to_the_same_log() {
    // Two instances are a designed state: the file opens with share
    // read/write, so interleaved appends both land.
    let dir = tempfile::tempdir().unwrap();
    let mut first = Logger::open(dir.path());
    let mut second = Logger::open(dir.path());
    first.log(&Record::records_lost(1));
    second.log(&Record::records_lost(2));
    first.log(&Record::shutdown_clean());
    let text = read_log(dir.path());
    assert_eq!(text.lines().count(), 3, "{text:?}");
}
