//! The panic hook (spec §14), verified by a real panic: this test binary
//! re-runs itself filtered to the trigger test, which installs the hook and
//! panics; the parent asserts the one `ERROR panic:` line landed. The hook
//! runs before unwinding and before abort alike, so what this harness
//! exercises is exactly what a `panic=abort` release runs.

#![cfg(windows)]

use std::fs;
use std::path::PathBuf;
use std::process::Command;

use pathmaster_platform::logwriter::LOG_FILE_NAME;

const TRIGGER_ENV: &str = "PATHMASTER_PANIC_HOOK_TEST_DIR";

#[test]
fn a_panic_reaches_the_log_as_one_error_line() {
    let dir = tempfile::tempdir().unwrap();
    let output = Command::new(std::env::current_exe().unwrap())
        .args(["panic_trigger", "--exact"])
        .env(TRIGGER_ENV, dir.path())
        .output()
        .expect("re-running the test binary");
    assert!(
        !output.status.success(),
        "the trigger child must actually panic",
    );

    let text = fs::read_to_string(dir.path().join(LOG_FILE_NAME)).expect("panic line written");
    let lines: Vec<&str> = text.lines().collect();
    assert_eq!(lines.len(), 1, "{text:?}");
    let line = lines[0];
    // `<RFC 3339 local+offset> ERROR panic: <message> (<file>:<line>)`
    assert_eq!(&line[10..11], "T", "{line:?}");
    assert!(
        line.contains(" ERROR panic: boom, but on purpose ("),
        "{line:?}"
    );
    assert!(line.contains("panic_hook.rs:"), "{line:?}");
    assert!(!line.contains("backtrace"), "{line:?}");
}

/// Not a test of its own: the child half of the harness above. Without the
/// env var (a normal test run) it does nothing and passes.
#[test]
fn panic_trigger() {
    let Some(dir) = std::env::var_os(TRIGGER_ENV) else {
        return;
    };
    pathmaster_platform::panic_hook::install(PathBuf::from(dir).join(LOG_FILE_NAME));
    panic!("boom, but on purpose");
}
