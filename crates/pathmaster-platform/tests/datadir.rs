//! Data Directory integration tests (spec §3, ADR-0002): the locate rule's
//! path mangling, the Writable / Read-only Data decision against real temp
//! directories, and the atomic-replace write helper. No mocks — the measured
//! hazards (junction-reporting `current_exe()`, verbatim canonicalize output)
//! are real filesystem behaviour.

#![cfg(windows)]

use std::fs;
use std::path::Path;

use pathmaster_platform::datadir::{
    decide, establish, locate, strip_verbatim_prefix, write_replace, DataDirState, ReadOnlyReason,
};

/// File-identity oracle: two paths name the same directory on disk. Used so
/// expected values need not reproduce the locate rule's own canonicalize +
/// strip pipeline (temp paths may carry 8.3 short names on some machines).
fn same_dir(a: &Path, b: &Path) -> bool {
    fs::canonicalize(a).unwrap() == fs::canonicalize(b).unwrap()
}

/// The names in a directory, for asserting nothing transient was left behind.
fn dir_names(dir: &Path) -> Vec<std::ffi::OsString> {
    fs::read_dir(dir)
        .unwrap()
        .map(|entry| entry.unwrap().file_name())
        .collect()
}

#[test]
fn verbatim_disk_prefix_is_stripped() {
    assert_eq!(
        strip_verbatim_prefix(Path::new(r"\\?\C:\Tools\PathMaster\PathMaster.exe")),
        Path::new(r"C:\Tools\PathMaster\PathMaster.exe"),
    );
}

#[test]
fn verbatim_unc_prefix_becomes_a_plain_unc_path() {
    assert_eq!(
        strip_verbatim_prefix(Path::new(r"\\?\UNC\server\share\PathMaster.exe")),
        Path::new(r"\\server\share\PathMaster.exe"),
    );
}

#[test]
fn locate_puts_data_beside_a_real_executable_in_plain_form() {
    let dir = tempfile::tempdir().unwrap();
    let exe = dir.path().join("PathMaster.exe");
    fs::write(&exe, b"stub").unwrap();

    let data = locate(&exe).unwrap();

    assert!(data.ends_with("data"));
    assert!(
        !data.to_string_lossy().starts_with(r"\\?\"),
        "canonicalize's verbatim prefix must be stripped: {}",
        data.display()
    );
    assert!(same_dir(data.parent().unwrap(), dir.path()));
}

/// The measured winget hazard: `current_exe()` reports a directory junction,
/// not its target — the naive rule would put `data\` in the shared `Links\`
/// directory. The locate rule must resolve through the junction.
#[test]
fn locate_resolves_a_junction_to_the_real_binary_directory() {
    let dir = tempfile::tempdir().unwrap();
    let real = dir.path().join("install");
    fs::create_dir(&real).unwrap();
    fs::write(real.join("PathMaster.exe"), b"stub").unwrap();
    let links = dir.path().join("Links");
    let status = std::process::Command::new("cmd")
        .args(["/C", "mklink", "/J"])
        .arg(&links)
        .arg(&real)
        .status()
        .unwrap();
    assert!(status.success(), "mklink /J failed");

    let data = locate(&links.join("PathMaster.exe")).unwrap();

    assert!(same_dir(data.parent().unwrap(), &real));
    assert_ne!(data, links.join("data"), "data\\ must not land in Links\\");
}

/// Resolution failure falls back to the unresolved path (spec §3) — the rule
/// still answers, it just cannot see through reparse points.
#[test]
fn locate_falls_back_to_the_unresolved_path_when_resolution_fails() {
    let exe = Path::new(r"C:\pathmaster-test-does-not-exist\PathMaster.exe");
    assert_eq!(
        locate(exe).unwrap(),
        Path::new(r"C:\pathmaster-test-does-not-exist\data"),
    );
}

/// An executable path with no parent leaves the Data Directory's own location
/// unknown — `locate` has no answer rather than a wrong one.
#[test]
fn locate_has_no_answer_for_a_parentless_path() {
    assert_eq!(locate(Path::new(r"C:\")), None);
}

/// The third reason's selection: no located directory (a failed
/// `current_exe()`, spec §3) is Read-only Data with the own-location-unknown
/// reason — the one reason that carries no directory at all.
#[test]
fn no_locate_answer_is_readonly_data_with_the_own_location_unknown_reason() {
    assert_eq!(
        decide(None),
        DataDirState::ReadOnly(ReadOnlyReason::OwnLocationUnknown),
    );
}

/// Which states name a directory is a property of the state, and the file
/// consumers ask it rather than re-deriving it: settings may still be readable
/// where nothing can be written, while an unknown own location has no
/// directory at all.
#[test]
fn every_state_but_an_unknown_own_location_names_a_directory_to_read_from() {
    let data = Path::new(r"C:\Tools\PathMaster\data");

    for state in [
        DataDirState::Writable(data.to_path_buf()),
        DataDirState::ReadOnly(ReadOnlyReason::CannotCreate(data.to_path_buf())),
        DataDirState::ReadOnly(ReadOnlyReason::NotWritable(data.to_path_buf())),
    ] {
        assert_eq!(state.dir(), Some(data), "{state:?}");
    }
    assert_eq!(
        DataDirState::ReadOnly(ReadOnlyReason::OwnLocationUnknown).dir(),
        None,
    );
}

/// Read-only Data closes *every* write path, renames and rotations included —
/// so exactly one state answers yes.
#[test]
fn only_writable_data_has_a_write_path() {
    let data = Path::new(r"C:\Tools\PathMaster\data");

    assert!(DataDirState::Writable(data.to_path_buf()).is_writable());
    for reason in [
        ReadOnlyReason::OwnLocationUnknown,
        ReadOnlyReason::CannotCreate(data.to_path_buf()),
        ReadOnlyReason::NotWritable(data.to_path_buf()),
    ] {
        assert!(
            !DataDirState::ReadOnly(reason.clone()).is_writable(),
            "{reason:?}"
        );
    }
}

#[test]
fn a_creatable_directory_establishes_writable_data_and_leaves_no_probe_behind() {
    let dir = tempfile::tempdir().unwrap();
    let data = dir.path().join("data");

    let state = establish(data.clone());

    assert_eq!(state, DataDirState::Writable(data.clone()));
    // The probe is transient (TC-file-structure): nothing may remain in a
    // Data Directory the app has only established.
    assert_eq!(dir_names(&data), Vec::<std::ffi::OsString>::new());
}

#[test]
fn a_directory_that_cannot_be_created_is_readonly_data_with_the_cannot_create_reason() {
    let dir = tempfile::tempdir().unwrap();
    // A file squatting on the Data Directory's path: `create_dir_all` fails.
    let data = dir.path().join("data");
    fs::write(&data, b"not a directory").unwrap();

    assert_eq!(
        establish(data.clone()),
        DataDirState::ReadOnly(ReadOnlyReason::CannotCreate(data)),
    );
}

/// Denies Everyone (`*S-1-1-0`) the create-file right on a directory for the
/// test's duration, restoring the DACL on drop so the temp dir can delete.
struct DenyCreateFile {
    dir: std::path::PathBuf,
}

impl DenyCreateFile {
    fn new(dir: &Path) -> Self {
        let status = std::process::Command::new("icacls")
            .arg(dir)
            .args(["/deny", "*S-1-1-0:(WD)"])
            .status()
            .unwrap();
        assert!(status.success(), "icacls /deny failed");
        DenyCreateFile {
            dir: dir.to_path_buf(),
        }
    }
}

impl Drop for DenyCreateFile {
    fn drop(&mut self) {
        let _ = std::process::Command::new("icacls")
            .arg(&self.dir)
            .args(["/remove:d", "*S-1-1-0"])
            .status();
    }
}

#[test]
fn an_unwritable_directory_is_readonly_data_with_the_not_writable_reason() {
    let dir = tempfile::tempdir().unwrap();
    let data = dir.path().join("data");
    fs::create_dir(&data).unwrap();
    let _deny = DenyCreateFile::new(&data);

    let state = establish(data.clone());

    // The reason is named and carries the directory where it was located —
    // Read-only Data never relocates (ADR-0002).
    assert_eq!(
        state,
        DataDirState::ReadOnly(ReadOnlyReason::NotWritable(data)),
    );
}

/// Both halves against a real executable, which is how `main` calls them: this
/// test binary's own directory is writable, so locating `data\` beside it and
/// deciding on the answer must give Writable Data.
#[test]
fn locate_and_decide_agree_on_a_writable_directory_beside_this_executable() {
    let exe = std::env::current_exe().unwrap();

    let state = decide(locate(&exe));

    match state {
        DataDirState::Writable(dir) => {
            assert!(dir.ends_with("data"));
            assert!(same_dir(dir.parent().unwrap(), exe.parent().unwrap()));
            // Empty after the transient probe, so this cleans up fully.
            fs::remove_dir(&dir).unwrap();
        }
        other => panic!("expected Writable beside the test exe, got {other:?}"),
    }
}

#[test]
fn write_replace_creates_a_file_that_did_not_exist() {
    let dir = tempfile::tempdir().unwrap();
    let target = dir.path().join("settings.json");

    write_replace(&target, b"{}").unwrap();

    assert_eq!(fs::read(&target).unwrap(), b"{}");
}

#[test]
fn write_replace_swaps_content_and_leaves_no_temp_file() {
    let dir = tempfile::tempdir().unwrap();
    let target = dir.path().join("settings.json");
    fs::write(&target, b"old").unwrap();

    write_replace(&target, b"new").unwrap();

    assert_eq!(fs::read(&target).unwrap(), b"new");
    assert_eq!(dir_names(dir.path()), vec!["settings.json"]);
}

#[test]
fn a_failed_replace_reports_the_error_and_cleans_its_temp_file_up() {
    let dir = tempfile::tempdir().unwrap();
    // A directory squatting on the target: the rename must fail.
    let target = dir.path().join("settings.json");
    fs::create_dir(&target).unwrap();

    assert!(write_replace(&target, b"new").is_err());

    assert_eq!(dir_names(dir.path()), vec!["settings.json"]);
    assert!(target.is_dir(), "the squatting target must be untouched");
}

#[test]
fn a_path_without_a_verbatim_prefix_is_untouched() {
    assert_eq!(
        strip_verbatim_prefix(Path::new(r"C:\Tools\PathMaster.exe")),
        Path::new(r"C:\Tools\PathMaster.exe"),
    );
    assert_eq!(
        strip_verbatim_prefix(Path::new(r"\\server\share\PathMaster.exe")),
        Path::new(r"\\server\share\PathMaster.exe"),
    );
}

#[test]
fn the_startup_log_state_names_the_reason_and_drops_the_location() {
    use pathmaster_core::logfmt::DataState as LogState;
    let dir = std::path::PathBuf::from(r"C:\somewhere\data");
    for (state, expected) in [
        (DataDirState::Writable(dir.clone()), LogState::Writable),
        (
            DataDirState::ReadOnly(ReadOnlyReason::OwnLocationUnknown),
            LogState::ReadOnlyOwnLocationUnknown,
        ),
        (
            DataDirState::ReadOnly(ReadOnlyReason::CannotCreate(dir.clone())),
            LogState::ReadOnlyCannotCreate,
        ),
        (
            DataDirState::ReadOnly(ReadOnlyReason::NotWritable(dir.clone())),
            LogState::ReadOnlyNotWritable,
        ),
    ] {
        assert_eq!(state.log_state(), expected, "{state:?}");
    }
}

#[test]
fn each_read_only_reason_names_a_registered_catalogue_string() {
    // The UI fills Announcement 7 ("Read-only: {reason}") with this msgid's
    // translation; the mapping lives beside the enum so a fourth reason cannot
    // appear without naming its string.
    use pathmaster_core::msgids;
    let dir = std::path::PathBuf::from(r"C:\somewhere\data");
    for (reason, expected) in [
        (
            ReadOnlyReason::OwnLocationUnknown,
            msgids::READONLY_REASON_OWN_LOCATION_UNKNOWN,
        ),
        (
            ReadOnlyReason::CannotCreate(dir.clone()),
            msgids::READONLY_REASON_CANNOT_CREATE,
        ),
        (
            ReadOnlyReason::NotWritable(dir.clone()),
            msgids::READONLY_REASON_NOT_WRITABLE,
        ),
    ] {
        assert_eq!(reason.catalogue_msgid(), expected, "{reason:?}");
        assert!(
            msgids::REGISTRY
                .iter()
                .any(|entry| entry.msgid == reason.catalogue_msgid()),
            "{reason:?} names a msgid the Catalogue does not hold"
        );
    }
}
