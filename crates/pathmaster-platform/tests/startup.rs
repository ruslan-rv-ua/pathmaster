//! The Run's properties, decided in one place (ADR-0010, impl ticket 20).
//!
//! Seven rules used to live in `main`, between calls that each had tests of
//! their own. These tests are about the glue: the whole startup sequence aimed
//! at a temporary Data Directory, temporary registry keys under
//! `HKCU\Software\PathMasterTest`, and **both** elevation answers — no
//! privilege, no real machine, and nothing here reads or writes the real
//! `PATH`.
//!
//! Two of the seven carry weight the rest do not. `data_writable && elevated`
//! is the `&&` ADR-0002 calls a trap when it is wrong — a Working Copy that can
//! never be applied — so it is asserted as a four-row table rather than as two
//! examples. And a Scope whose startup read *fails* becomes an empty
//! **non-writable** Session: a rule the spec never states, invented by impl
//! ticket 08 on the grounds that nothing may be written over a value that was
//! never read, and untested until now.

#![cfg(windows)]

use std::fs;
use std::panic;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

use pathmaster_core::language::{Language, SystemLanguage};
use pathmaster_core::logfmt::{DataState, FailureCause, Record};
use pathmaster_core::session::{Scope, Session, ValueType};
use pathmaster_core::settings::{Parsed, SettingsFile};
use pathmaster_platform::datadir::ReadOnlyReason;
use pathmaster_platform::logwriter::LOG_FILE_NAME;
use pathmaster_platform::registry::{Hive, RawValue, ScopeKey};
use pathmaster_platform::settings::{BAD_FILE_NAME, FILE_NAME};
use pathmaster_platform::startup::{self, Decisions};

const TEST_ROOT: &str = r"Software\PathMasterTest";

/// One test's private registry subkey, deleted with the shared parent when the
/// test finishes. A startup read never creates a key, so most of these tests
/// leave nothing behind to delete — the guard is for the ones that plant.
struct TestKey {
    path: String,
}

impl TestKey {
    fn new(name: &str) -> TestKey {
        TestKey {
            path: format!(r"{TEST_ROOT}\{name}-{}", std::process::id()),
        }
    }

    fn key(&self) -> ScopeKey {
        ScopeKey::at(Hive::CurrentUser, &self.path, "Path")
    }

    /// Plants a value the run did not write: what the registry held when
    /// startup read it.
    fn plant(&self, value_type: ValueType, value: &str) {
        self.key()
            .write(value_type, value)
            .expect("a planted value");
    }

    /// Plants a value the adapter cannot read at all. `REG_BINARY` is the
    /// unsupported type `tests/registry.rs` already uses, and it is the
    /// cheapest real read failure a test can arrange without a privilege.
    fn plant_unreadable(&self) {
        let hkcu = winreg::RegKey::predef(winreg::enums::HKEY_CURRENT_USER);
        let (key, _) = hkcu.create_subkey(&self.path).unwrap();
        key.set_raw_value(
            "Path",
            &winreg::RegValue {
                vtype: winreg::enums::RegType::REG_BINARY,
                bytes: vec![1, 2, 3].into(),
            },
        )
        .unwrap();
    }
}

impl Drop for TestKey {
    fn drop(&mut self) {
        let hkcu = winreg::RegKey::predef(winreg::enums::HKEY_CURRENT_USER);
        let _ = hkcu.delete_subkey_all(&self.path);
        // The shared parent only deletes once the last concurrent test is done.
        let _ = hkcu.delete_subkey(TEST_ROOT);
    }
}

/// A temporary Data Directory's parent and one temporary registry key per
/// Scope — everything `decide` is allowed to reach.
struct World {
    home: tempfile::TempDir,
    user: TestKey,
    system: TestKey,
}

impl World {
    fn new(name: &str) -> World {
        World {
            home: tempfile::tempdir().unwrap(),
            user: TestKey::new(&format!("{name}-user")),
            system: TestKey::new(&format!("{name}-system")),
        }
    }

    /// Where `data\` goes: what `locate` would have answered for an executable
    /// sitting in this world.
    fn located(&self) -> PathBuf {
        self.home.path().join("data")
    }

    fn settings_file(&self) -> PathBuf {
        self.located().join(FILE_NAME)
    }

    /// Writes `settings.json` before the run, which means creating `data\`
    /// first — exactly the state a second run starts from.
    fn write_settings(&self, text: &str) {
        fs::create_dir_all(self.located()).unwrap();
        fs::write(self.settings_file(), text).unwrap();
    }

    fn decide(&self, elevated: bool) -> Decisions {
        self.decide_from(Some(self.located()), elevated, SystemLanguage::Other)
    }

    fn decide_from(
        &self,
        located: Option<PathBuf>,
        elevated: bool,
        system: SystemLanguage,
    ) -> Decisions {
        // `decide` installs the process-wide panic hook whenever the Run has a
        // log (rule two), and that hook writes to the log instead of printing —
        // left in place it would swallow libtest's own report for every test
        // that ran after the first. The harness's hook goes back the moment
        // `decide` returns, and the lock keeps two parallel tests from trading
        // hooks mid-call. That the hook is installed at all is asserted by the
        // two child-process tests below, where clobbering it is the point.
        let _serialised = HOOK.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
        let harness_hook = panic::take_hook();
        let decided = startup::decide(
            located,
            elevated,
            system,
            &self.user.key(),
            &self.system.key(),
        );
        panic::set_hook(harness_hook);
        decided
    }
}

static HOOK: Mutex<()> = Mutex::new(());

/// The `INFO startup:` line this build earns. The version is the workspace's,
/// which is the binary's.
fn startup_record(elevated: bool, data: DataState, language: Language) -> Record {
    Record::startup(env!("CARGO_PKG_VERSION"), elevated, data, language.code())
}

fn raws(session: &Session) -> Vec<&str> {
    session.entries().iter().map(|entry| entry.raw()).collect()
}

// ---------------------------------------------------------------------------
// Rule one: Read-only Data is a Run without a log.
// ---------------------------------------------------------------------------

#[test]
fn writable_data_gives_the_run_a_log_in_its_data_directory() {
    let world = World::new("log-writable");

    let decided = world.decide(false);

    assert_eq!(decided.run.data_dir(), Some(world.located().as_path()));
    assert_eq!(
        decided.run.log_path(),
        Some(world.located().join(LOG_FILE_NAME).as_path()),
    );
    assert_eq!(decided.readonly, None, "there is no reason to carry");
}

/// Rule one's sharper half (spec §14): an unopenable log is a Run without a
/// log, **never** Read-only Data. A directory wearing the log's name is the
/// cheapest way to arrange one — `data\` is still perfectly writable, so the
/// two decisions must come apart.
#[test]
fn a_log_that_cannot_be_opened_does_not_make_the_run_read_only() {
    let world = World::new("log-unopenable");
    fs::create_dir_all(world.located().join(LOG_FILE_NAME)).unwrap();

    let decided = world.decide(false);

    assert_eq!(decided.run.log_path(), None, "a Run without a log");
    assert_eq!(decided.readonly, None, "but not a Read-only Data Run");
    assert_eq!(decided.run.data_dir(), Some(world.located().as_path()));
    // Which is to say the startup line it earned says so, and is dropped.
    assert_eq!(
        decided.records,
        vec![startup_record(
            false,
            DataState::Writable,
            Language::English
        )],
    );
}

#[test]
fn read_only_data_is_a_run_without_a_log_and_writes_nothing() {
    let world = World::new("log-readonly");
    // A file squatting on `data\`: `create_dir_all` fails, so the run is
    // Read-only Data with a directory it can name but not create.
    fs::write(world.located(), b"not a directory").unwrap();

    let decided = world.decide(false);

    assert_eq!(decided.run.log_path(), None);
    // The directory it names is still the located one — Read-only Data never
    // relocates (ADR-0002).
    assert_eq!(decided.run.data_dir(), Some(world.located().as_path()));
    assert_eq!(
        fs::read(world.located()).unwrap(),
        b"not a directory",
        "nothing may be written in Read-only Data",
    );
    assert_eq!(
        decided.readonly,
        Some(ReadOnlyReason::CannotCreate(world.located())),
        "the reason survives to the UI",
    );
}

#[test]
fn an_unknown_own_location_is_a_run_with_neither_a_log_nor_a_directory() {
    let world = World::new("log-nowhere");

    let decided = world.decide_from(None, false, SystemLanguage::Other);

    assert_eq!(decided.run.log_path(), None);
    assert_eq!(decided.run.data_dir(), None);
    assert_eq!(
        decided.readonly,
        Some(ReadOnlyReason::OwnLocationUnknown),
        "the reason survives to the UI",
    );
}

// ---------------------------------------------------------------------------
// Rule three: the startup record precedes the settings records.
// Rule four: `Source`'s three arms decide one dialog flag and the WARN records.
// ---------------------------------------------------------------------------

#[test]
fn a_first_run_logs_the_startup_line_and_nothing_else() {
    let world = World::new("settings-absent");

    let decided = world.decide(false);

    assert_eq!(
        decided.records,
        vec![startup_record(
            false,
            DataState::Writable,
            Language::English
        )],
        "an absent settings.json is a first run, not a failure",
    );
    assert!(!decided.settings_unreadable, "and costs no dialog");
    assert_eq!(decided.settings, SettingsFile::defaults());
}

#[test]
fn rejected_fields_are_warned_about_in_order_under_the_startup_line() {
    let world = World::new("settings-rejected");
    let text = r#"{"language": "klingon", "maxBackups": 2.5}"#;
    world.write_settings(text);

    let decided = world.decide(false);

    let Parsed::Readable { file, rejected } = SettingsFile::parse(text) else {
        panic!("the fixture parses");
    };
    assert_eq!(rejected.len(), 2, "the fixture rejects both fields");
    let mut expected = vec![startup_record(
        false,
        DataState::Writable,
        Language::English,
    )];
    expected.extend(rejected.iter().map(|rejection| rejection.record()));
    assert_eq!(decided.records, expected, "the startup line comes first");
    assert!(
        !decided.settings_unreadable,
        "a bad field is noise, not a dialog",
    );
    // The backup budget travels with the settings, not with the Run: the
    // window holds this file and each Apply reads the budget off it (ADR-0010).
    assert_eq!(decided.settings, file);
}

#[test]
fn an_unreadable_settings_file_costs_one_dialog_one_warn_and_its_name() {
    let world = World::new("settings-unreadable");
    world.write_settings("{ this is not json");

    let decided = world.decide(false);

    assert!(decided.settings_unreadable, "the user is owed a dialog");
    assert_eq!(
        decided.records,
        vec![
            startup_record(false, DataState::Writable, Language::English),
            Record::settings_unreadable(true),
        ],
    );
    assert!(!world.settings_file().exists(), "set aside, not left");
    assert!(world.located().join(BAD_FILE_NAME).exists());
    assert_eq!(decided.settings, SettingsFile::defaults());
}

/// Read-only Data still reads its settings — a run that cannot write its
/// directory can still obey the language the user chose — but performs no
/// write, and the set-aside is a write.
#[test]
fn read_only_data_reads_its_settings_and_leaves_a_bad_file_in_place() {
    let world = World::new("settings-readonly");
    world.write_settings("{ this is not json");
    let _deny = DenyCreateFile::new(&world.located());

    let decided = world.decide(false);

    assert!(decided.settings_unreadable, "the dialog is owed either way");
    assert!(
        world.settings_file().exists(),
        "their file stays exactly where they left it",
    );
    assert!(!world.located().join(BAD_FILE_NAME).exists());
    assert_eq!(
        decided.records,
        vec![
            startup_record(false, DataState::ReadOnlyNotWritable, Language::English),
            Record::settings_unreadable(false),
        ],
        "a run without a log still earns its records; nothing writes them",
    );
    assert_eq!(
        decided.readonly,
        Some(ReadOnlyReason::NotWritable(world.located())),
    );
}

// ---------------------------------------------------------------------------
// The Interface Language: the stored choice, and the system behind it.
// ---------------------------------------------------------------------------

#[test]
fn the_stored_choice_beats_the_system_language_and_the_startup_line_says_so() {
    let world = World::new("language-stored");
    world.write_settings(r#"{"language": "uk"}"#);

    let decided = world.decide_from(Some(world.located()), false, SystemLanguage::Other);

    assert_eq!(decided.language, Language::Ukrainian);
    assert_eq!(
        decided.records,
        vec![startup_record(
            false,
            DataState::Writable,
            Language::Ukrainian
        )],
    );
}

#[test]
fn auto_follows_the_system_language() {
    let world = World::new("language-auto");
    world.write_settings(r#"{"language": "auto"}"#);

    let decided = world.decide_from(Some(world.located()), false, SystemLanguage::Ukrainian);

    assert_eq!(decided.language, Language::Ukrainian);
}

// ---------------------------------------------------------------------------
// Rule five: User writes with the Run, System also needs elevation.
// ---------------------------------------------------------------------------

/// ADR-0002 calls a wrong answer here a trap: a Working Copy the user may edit
/// and can never apply. All four rows, both elevation answers.
#[test]
fn user_writes_with_the_run_and_system_also_needs_elevation() {
    let world = World::new("writability");

    for (data_writable, elevated, user_writable, system_writable) in [
        (true, false, true, false),
        (true, true, true, true),
        (false, false, false, false),
        (false, true, false, false),
    ] {
        let located = data_writable.then(|| world.located());
        let decided = world.decide_from(located, elevated, SystemLanguage::Other);

        let row = format!("data_writable: {data_writable}, elevated: {elevated}");
        assert_eq!(decided.user.session.writable(), user_writable, "{row}");
        assert_eq!(decided.system.session.writable(), system_writable, "{row}");
    }
}

// ---------------------------------------------------------------------------
// Rule seven: a Scope whose read fails is empty and non-writable.
// ---------------------------------------------------------------------------

#[test]
fn a_scope_that_reads_carries_its_value_and_the_bytes_it_was_read_from() {
    let world = World::new("scope-read");
    world
        .user
        .plant(ValueType::RegExpandSz, r"C:\bin;%SystemRoot%");
    world.system.plant(ValueType::RegSz, r"C:\literal");

    let decided = world.decide(true);

    assert_eq!(raws(&decided.user.session), vec![r"C:\bin", "%SystemRoot%"]);
    assert_eq!(decided.user.session.value_type(), ValueType::RegExpandSz);
    assert_eq!(raws(&decided.system.session), vec![r"C:\literal"]);
    assert_eq!(decided.system.session.value_type(), ValueType::RegSz);
    // The raw value is kept beside the Session: external-change detection
    // compares `(vtype, bytes)`, and a decoded copy would miss a change past
    // the first NUL (spec §4).
    assert_eq!(decided.user.last_read, world.user.key().read().unwrap());
    assert_ne!(decided.user.last_read, RawValue::Absent);
}

/// The rule impl ticket 08 invented and nothing has asserted since: a read that
/// *failed* is not an empty Scope. Nothing may be written over a value that was
/// never read, so the Session is empty **and** non-writable — even in a run
/// that had every right to write it.
#[test]
fn a_scope_whose_read_fails_is_empty_and_never_writable() {
    let world = World::new("scope-failed");
    world.user.plant_unreadable();
    world.system.plant(ValueType::RegExpandSz, r"C:\bin");

    let decided = world.decide(true);

    assert!(raws(&decided.user.session).is_empty());
    assert!(
        !decided.user.session.writable(),
        "a Writable-Data elevated run still may not write what it could not read",
    );
    assert_eq!(
        decided.user.last_read,
        RawValue::Absent,
        "and its last-read value is the one nothing ever compares",
    );
    // The other Scope is untouched by its neighbour's failure.
    assert!(decided.system.session.writable());
    assert_eq!(raws(&decided.system.session), vec![r"C:\bin"]);
    assert_eq!(
        decided.records,
        vec![
            startup_record(true, DataState::Writable, Language::English),
            Record::scope_read_failed(Scope::User, FailureCause::UnsupportedType { vtype: 3 }),
        ],
        "the log line is the developer's only witness",
    );
}

// ---------------------------------------------------------------------------
// Rule two: the panic hook installs only where there is a log path.
//
// Asserted the way `tests/panic_hook.rs` asserts the hook itself — by a real
// panic in a child process, because the hook is process-wide and what it
// replaces is libtest's own reporting.
// ---------------------------------------------------------------------------

const TRIGGER_ENV: &str = "PATHMASTER_STARTUP_PANIC_HOOK_TEST_HOME";

/// Re-runs this test binary filtered to the trigger below, with `home` as the
/// world it starts up in, and returns everything the child said.
fn panic_in_a_child(home: &Path) -> String {
    let output = std::process::Command::new(std::env::current_exe().unwrap())
        .args(["panic_trigger", "--exact"])
        .env(TRIGGER_ENV, home)
        .output()
        .expect("re-running the test binary");
    assert!(
        !output.status.success(),
        "the trigger child must actually panic",
    );
    String::from_utf8_lossy(&output.stdout).into_owned() + &String::from_utf8_lossy(&output.stderr)
}

#[test]
fn a_run_with_a_log_installs_the_panic_hook_against_it() {
    let home = tempfile::tempdir().unwrap();

    let said = panic_in_a_child(home.path());

    let log = fs::read_to_string(home.path().join("data").join(LOG_FILE_NAME))
        .expect("the panic line reached the log");
    assert!(
        log.contains(" ERROR panic: boom, but on purpose ("),
        "{log:?}"
    );
    assert!(
        !said.contains("panicked at"),
        "the hook replaces the default one, which is why it must not be left \
         installed in this binary: {said}",
    );
}

#[test]
fn a_run_without_a_log_leaves_the_default_hook_alone() {
    let home = tempfile::tempdir().unwrap();
    // A file squatting on `data\` — Read-only Data, so there is no log to
    // install a hook against.
    fs::write(home.path().join("data"), b"not a directory").unwrap();

    let said = panic_in_a_child(home.path());

    assert!(
        said.contains("panicked at"),
        "nothing was installed over the default hook: {said}",
    );
}

/// Not a test of its own: the child half of the two above. Without the env var
/// (a normal test run) it does nothing and passes.
#[test]
fn panic_trigger() {
    let Some(home) = std::env::var_os(TRIGGER_ENV) else {
        return;
    };
    // A registry key that was never created: a startup read of one is Absent,
    // and reading never creates anything to clean up.
    let key = TestKey::new("panic-trigger").key();
    let _ = startup::decide(
        Some(PathBuf::from(home).join("data")),
        false,
        SystemLanguage::Other,
        &key,
        &key,
    );
    panic!("boom, but on purpose");
}

/// Denies Everyone (`*S-1-1-0`) the create-file right on a directory for the
/// test's duration, restoring the DACL on drop so the temp dir can delete.
/// Lifted from `tests/datadir.rs`, where the same trick makes `establish`
/// answer `NotWritable` without a privilege.
struct DenyCreateFile {
    dir: PathBuf,
}

impl DenyCreateFile {
    fn new(dir: &Path) -> DenyCreateFile {
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

// ---------------------------------------------------------------------------
// The Run keeps the facts it was decided from (ADR-0010, ticket 17).
// ---------------------------------------------------------------------------

/// Whether this process is elevated is a property of the Run, decided once and
/// held for the window: the elevated instance must title itself and disable
/// its own way back into elevation, and rederiving the answer from a Session's
/// writability would misread the Read-only Data and failed-read runs (spec §9).
#[test]
fn the_run_keeps_the_elevation_answer_it_was_decided_from() {
    let world = World::new("run-elevated");

    assert!(!world.decide(false).run.elevated());
    assert!(world.decide(true).run.elevated());
}
