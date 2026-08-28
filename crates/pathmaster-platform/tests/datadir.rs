//! Data Directory integration tests (spec §3, ADR-0002; v0.2.0 §10): the locate
//! rule's path mangling, `--data-dir`'s substitution of that rule, the Writable
//! / Read-only Data decision against real temp directories, the atomic-replace
//! write helper, and TC-file-structure's whole inventory driven through every
//! writer that puts something in `data\`. No mocks — the measured hazards
//! (junction-reporting `current_exe()`, verbatim canonicalize output) are real
//! filesystem behaviour.

#![cfg(windows)]

use std::ffi::OsStr;
use std::fs;
use std::path::{Path, PathBuf};

use pathmaster_platform::datadir::{
    decide, establish, locate, locate_override, strip_verbatim_prefix, write_replace, DataDirState,
    Location, ReadOnlyReason,
};

/// The default road's locate answer, as `main` assembles it.
fn beside_exe(dir: PathBuf) -> Location {
    Location::BesideExe(dir)
}

/// One `--data-dir` value, resolved against a current directory the test picks.
fn overridden(value: &str, cwd: &str) -> Location {
    locate_override(OsStr::new(value), Path::new(cwd))
}

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
        decide(Location::OwnLocationUnknown),
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
        DataDirState::ReadOnly(ReadOnlyReason::OverrideUnusable(Some(data.to_path_buf()))),
    ] {
        assert_eq!(state.dir(), Some(data), "{state:?}");
    }
    for state in [
        DataDirState::ReadOnly(ReadOnlyReason::OwnLocationUnknown),
        DataDirState::ReadOnly(ReadOnlyReason::OverrideUnusable(None)),
    ] {
        assert_eq!(state.dir(), None, "{state:?}");
    }
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
        ReadOnlyReason::OverrideUnusable(Some(data.to_path_buf())),
        ReadOnlyReason::OverrideUnusable(None),
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

// ---------------------------------------------------------------------------
// `--data-dir`: the locate step substituted, and nothing else (v0.2.0 §10).
// ---------------------------------------------------------------------------

/// An absolute value is taken as it stands — no canonicalisation, so no `\\?\`
/// prefix rides onto the command line the elevated instance is handed.
#[test]
fn an_absolute_override_is_used_as_it_stands() {
    assert_eq!(
        overridden(r"D:\PathMaster data", r"C:\somewhere\else"),
        Location::Override(PathBuf::from(r"D:\PathMaster data")),
    );
}

/// A relative value resolves against the current directory, which is what a
/// shell user means and what every verifiable precedent does.
#[test]
fn a_relative_override_resolves_against_the_current_directory() {
    assert_eq!(
        overridden("pm-data", r"C:\work"),
        Location::Override(PathBuf::from(r"C:\work\pm-data")),
    );
    // A root-relative path keeps the current directory's drive, the way
    // Windows reads it.
    assert_eq!(
        overridden(r"\pm-data", r"D:\work\deep"),
        Location::Override(PathBuf::from(r"D:\pm-data")),
    );
}

/// The two artifacts Windows' own parsing leaves on a quoted path —
/// `--data-dir "C:\x\"` arrives as `C:\x"` — stripped before resolution, and in
/// that order, so a value carrying both still comes out clean.
#[test]
fn the_quoting_artifacts_are_stripped_before_the_path_is_resolved() {
    for value in [
        r"C:\pm-data",
        r#"C:\pm-data""#,
        r"C:\pm-data\",
        r"C:\pm-data//",
        r#"C:\pm-data\""#,
    ] {
        assert_eq!(
            overridden(value, r"C:\work"),
            Location::Override(PathBuf::from(r"C:\pm-data")),
            "value: {value:?}"
        );
    }
}

/// Never past the root: `C:\` is a directory, and `C:` is a drive-relative path
/// that names a different one. Stripping that separator would point the Run
/// somewhere the user did not.
#[test]
fn the_separator_strip_never_eats_a_root() {
    assert_eq!(
        overridden(r"C:\", r"D:\work"),
        Location::Override(PathBuf::from(r"C:\")),
    );
}

/// A switch with nothing that resolves is a **broken override**: Read-only Data
/// through the fourth reason, never a fallback to the default `data\`.
///
/// The drive-relative `C:foo` is one of them. It names a current directory *per
/// drive* that only the OS knows, and this application may not guess at where
/// it writes — a guess is not a pointing.
#[test]
fn a_value_that_resolves_to_nothing_is_a_broken_override() {
    for value in ["", r#"""#, r#""""#, "C:foo"] {
        assert_eq!(
            overridden(value, r"C:\work"),
            Location::BrokenOverride,
            "value: {value:?}"
        );
    }
    assert_eq!(
        decide(Location::BrokenOverride),
        DataDirState::ReadOnly(ReadOnlyReason::OverrideUnusable(None)),
    );
}

/// The substitution is of the locate step **only**: a missing directory is
/// `create_dir_all`-created there exactly as `data\` would be beside the
/// executable, transient probe and all.
#[test]
fn an_override_creates_its_directory_like_the_default_one() {
    let dir = tempfile::tempdir().unwrap();
    let target = dir.path().join("fresh").join("pm-data");

    let state = decide(locate_override(target.as_os_str(), Path::new(r"C:\unused")));

    assert_eq!(state, DataDirState::Writable(target.clone()));
    assert_eq!(dir_names(&target), Vec::<std::ffi::OsString>::new());
}

/// A target that cannot be used lands the Run in Read-only Data through the
/// **fourth reason**, which names the switch — and keeps the directory, because
/// a `settings.json` may still be readable there. Never the default `data\`.
#[test]
fn an_unusable_override_names_the_switch_and_keeps_its_directory() {
    let dir = tempfile::tempdir().unwrap();
    // A file squatting on the path: `create_dir_all` fails, exactly as it does
    // on the default road.
    let target = dir.path().join("pm-data");
    fs::write(&target, b"not a directory").unwrap();

    let state = decide(Location::Override(target.clone()));

    assert_eq!(
        state,
        DataDirState::ReadOnly(ReadOnlyReason::OverrideUnusable(Some(target.clone()))),
    );
    assert_eq!(state.dir(), Some(target.as_path()));
    assert!(!state.is_writable());
}

/// An override whose directory exists but refuses a write is the same one
/// reason — one thing to say, one thing to do — and still keeps its directory.
#[test]
fn an_unwritable_override_is_the_same_reason_as_an_uncreatable_one() {
    let dir = tempfile::tempdir().unwrap();
    let target = dir.path().join("pm-data");
    fs::create_dir(&target).unwrap();
    let _deny = DenyCreateFile::new(&target);

    assert_eq!(
        decide(Location::Override(target.clone())),
        DataDirState::ReadOnly(ReadOnlyReason::OverrideUnusable(Some(target))),
    );
}

/// The same failure on the default road keeps its own reason: the switch's
/// reason exists to tell the two locations apart, so it may not leak onto a Run
/// that was never pointed anywhere.
#[test]
fn the_default_road_keeps_its_own_reasons() {
    let dir = tempfile::tempdir().unwrap();
    let data = dir.path().join("data");
    fs::write(&data, b"not a directory").unwrap();

    assert_eq!(
        decide(beside_exe(data.clone())),
        DataDirState::ReadOnly(ReadOnlyReason::CannotCreate(data)),
    );
}

/// Only a resolved override has a path to carry beyond the decision — which is
/// what keeps the audited exception to the path prohibition narrow, and what
/// tells a relaunch whether it has an override to re-serialize at all.
#[test]
fn only_a_resolved_override_names_a_path_beyond_the_decision() {
    let dir = PathBuf::from(r"D:\pm-data");

    assert_eq!(
        Location::Override(dir.clone()).override_path(),
        Some(dir.as_path())
    );
    for location in [
        Location::BesideExe(dir.clone()),
        Location::OwnLocationUnknown,
        Location::BrokenOverride,
    ] {
        assert_eq!(location.override_path(), None, "{location:?}");
    }

    assert!(Location::Override(dir.clone()).is_override());
    assert!(Location::BrokenOverride.is_override());
    assert!(!Location::BesideExe(dir).is_override());
    assert!(!Location::OwnLocationUnknown.is_override());
}

/// Both halves against a real executable, which is how `main` calls them: this
/// test binary's own directory is writable, so locating `data\` beside it and
/// deciding on the answer must give Writable Data.
#[test]
fn locate_and_decide_agree_on_a_writable_directory_beside_this_executable() {
    let exe = std::env::current_exe().unwrap();

    let state = decide(locate(&exe).map(beside_exe).unwrap());

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
        (
            DataDirState::ReadOnly(ReadOnlyReason::OverrideUnusable(Some(dir.clone()))),
            LogState::ReadOnlyOverrideUnusable,
        ),
        (
            DataDirState::ReadOnly(ReadOnlyReason::OverrideUnusable(None)),
            LogState::ReadOnlyOverrideUnusable,
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
        (
            ReadOnlyReason::OverrideUnusable(Some(dir.clone())),
            msgids::READONLY_REASON_OVERRIDE_UNUSABLE,
        ),
        (
            ReadOnlyReason::OverrideUnusable(None),
            msgids::READONLY_REASON_OVERRIDE_UNUSABLE,
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

/// TC-file-structure's whole `data\` inventory, **driven rather than restated**
/// (spec §3, grown by one file in v0.2.0 §15).
///
/// The rule is "nothing else, anywhere", and until now nothing said so in one
/// place: each writer had tests for its own file and none of them could see a
/// fourth appear beside it. This performs every write the application makes
/// into the Data Directory and then reads the directory back, so a module that
/// starts leaving something behind fails here — rather than in the Process
/// Monitor session of Release Checklist E2, months later, on a released build.
///
/// `data\backups\*.json` and its transient `.tmp` are `snapshots.rs`'s to gate;
/// what this fixes is the level above them.
#[test]
fn a_data_directory_holds_the_file_structure_and_nothing_else() {
    use pathmaster_core::settings::SettingsFile;
    use pathmaster_platform::logwriter::{Logger, LOG_FILE_NAME, OLD_FILE_NAME};
    use pathmaster_platform::{help, settings, snapshots};

    let home = tempfile::tempdir().unwrap();
    let data = home.path().join("data");
    assert_eq!(
        establish(data.clone()),
        DataDirState::Writable(data.clone())
    );

    // settings.json.bad, made the only way it is ever made: by a read that
    // could not parse what was there. The set-aside leaves no settings.json.
    fs::write(data.join(settings::FILE_NAME), b"{ not json").unwrap();
    settings::read(&DataDirState::Writable(data.clone()));
    settings::write(&data, &SettingsFile::defaults()).unwrap();

    snapshots::ensure_folder(&data);

    // Rotation happens at open, over 1 MB, and leaves both generations behind.
    fs::write(data.join(LOG_FILE_NAME), vec![b'x'; 1_048_577]).unwrap();
    let _logger = Logger::open(&data);

    // The v0.2.0 addition. Written on every F1, under one name whatever
    // language it is in.
    help::write_page(&data, b"<!doctype html>").unwrap();

    let mut found: Vec<String> = dir_names(&data)
        .iter()
        .map(|name| name.to_string_lossy().into_owned())
        .collect();
    let mut inventory: Vec<String> = [
        settings::FILE_NAME,
        settings::BAD_FILE_NAME,
        snapshots::DIR_NAME,
        LOG_FILE_NAME,
        OLD_FILE_NAME,
        help::FILE_NAME,
    ]
    .iter()
    .map(|name| (*name).to_owned())
    .collect();
    found.sort();
    inventory.sort();
    assert_eq!(found, inventory);
}
