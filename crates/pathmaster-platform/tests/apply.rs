//! The Apply Run at the crate boundary (spec §5 FR-apply, §4, §7, §8, §9;
//! ADR-0008, ticket impl-13).
//!
//! These tests are not about which rule fired. They are about **whether the
//! order held when something went wrong**: was the Snapshot still there, was
//! the registry still untouched, was the backup the run had just taken still
//! on disk. That is why they run against the live registry under a temporary
//! key and a temporary Data Directory rather than against a mock — against a
//! mock they would assert nothing.
//!
//! The Working Copies arrive by value and no Session is in sight, which is the
//! taxonomy's first invariant made unrepresentable: this module is handed no
//! Baseline to move (ADR-0008). What each test can therefore check is the two
//! things a failure must leave alone — the file and the value — plus the
//! outcome the window is handed to move the Baseline with.

#![cfg(windows)]

use std::cell::RefCell;
use std::fs;
use std::io;
use std::path::Path;

use pathmaster_core::logfmt::Timestamp;
use pathmaster_core::msgids;
use pathmaster_core::normalize::Environment;
use pathmaster_core::session::{Scope, ValueType};
use pathmaster_core::snapshot::{Captured, Decoded, Snapshot, SnapshotName};
use pathmaster_platform::apply::{
    self, ApplyRun, Ask, ExternalChange, Failure, ScopeInput, ScopeOutcome,
};
use pathmaster_platform::registry::{Hive, RawValue, RegistryError, ScopeKey};
use pathmaster_platform::snapshots;

const TEST_ROOT: &str = r"Software\PathMasterTest";

/// One test's private registry subkey, deleted with the shared parent when the
/// test finishes.
struct TestKey {
    path: String,
}

impl TestKey {
    fn new(name: &str) -> Self {
        TestKey {
            path: format!(r"{TEST_ROOT}\{name}-{}", std::process::id()),
        }
    }

    fn key(&self) -> ScopeKey {
        ScopeKey::at(Hive::CurrentUser, &self.path, "Path")
    }

    /// Plants a value the run did not write — a startup read's worth of state,
    /// or an external edit made while the Session was open.
    fn plant(&self, value_type: ValueType, value: &str) -> RawValue {
        self.key()
            .write(value_type, value)
            .expect("a planted value");
        self.key().read().expect("the planted value reads back")
    }
}

impl Drop for TestKey {
    fn drop(&mut self) {
        let hkcu = winreg::RegKey::predef(winreg::enums::HKEY_CURRENT_USER);
        let _ = hkcu.delete_subkey_all(&self.path);
        let _ = hkcu.delete_subkey(TEST_ROOT);
    }
}

/// A temporary Data Directory and one temporary registry key per Scope.
struct World {
    data: tempfile::TempDir,
    user: TestKey,
    system: TestKey,
}

impl World {
    fn new(name: &str) -> World {
        World {
            data: tempfile::tempdir().unwrap(),
            user: TestKey::new(&format!("{name}-user")),
            system: TestKey::new(&format!("{name}-system")),
        }
    }

    fn data_dir(&self) -> &Path {
        self.data.path()
    }

    fn backups(&self) -> std::path::PathBuf {
        snapshots::dir(self.data_dir())
    }

    /// The Snapshots on disk, oldest first.
    fn snapshots(&self) -> Vec<SnapshotName> {
        snapshots::listing(&self.backups()).expect("a listing")
    }

    /// The one Snapshot on disk, decoded. Fails loudly if there is not exactly
    /// one — "a backup was taken" and "one backup was taken" are different
    /// claims.
    fn only_snapshot(&self) -> Snapshot {
        let listing = self.snapshots();
        assert_eq!(listing.len(), 1, "exactly one Snapshot: {listing:?}");
        let text = fs::read_to_string(self.backups().join(listing[0].file_name())).unwrap();
        match Snapshot::decode(&text) {
            Decoded::Valid(snapshot) => snapshot,
            Decoded::Corrupted => panic!("the run wrote a Corrupted Snapshot: {text}"),
        }
    }
}

/// The clock, fixed — which is the whole reason it is a parameter (ADR-0008).
fn at(second: u8) -> Timestamp {
    Timestamp {
        year: 2026,
        month: 8,
        day: 21,
        hour: 14,
        minute: 32,
        second,
        offset_minutes: 180,
    }
}

/// The environment expansion reads. Empty: the only thing this crate's tests
/// ask of it is the merged length, and a defined variable would make that
/// number depend on the machine.
struct Env;

impl Environment for Env {
    fn lookup(&self, _name: &str) -> Option<String> {
        None
    }
}

/// One question the run asked.
#[derive(Debug, PartialEq, Eq)]
enum Asked {
    ExternalChange(Scope),
    CmdLimit(usize),
    HardCap(usize),
}

/// The tests' adapter for the three questions: scripted answers, and a record
/// of what was actually asked — because "asked nothing" is as much a rule as
/// any answer (spec §7: a length within every threshold has no dialog).
struct Scripted {
    external_change: ExternalChange,
    cmd_limit: bool,
    asked: RefCell<Vec<Asked>>,
}

impl Scripted {
    fn new() -> Scripted {
        Scripted {
            external_change: ExternalChange::Overwrite,
            cmd_limit: true,
            asked: RefCell::new(Vec::new()),
        }
    }

    fn answering(external_change: ExternalChange) -> Scripted {
        Scripted {
            external_change,
            ..Scripted::new()
        }
    }

    fn refusing_the_cmd_limit() -> Scripted {
        Scripted {
            cmd_limit: false,
            ..Scripted::new()
        }
    }

    fn asked(&self) -> Vec<Asked> {
        self.asked.take()
    }
}

impl Ask for Scripted {
    fn external_change(&self, scope: Scope) -> ExternalChange {
        self.asked.borrow_mut().push(Asked::ExternalChange(scope));
        self.external_change
    }

    fn cmd_limit(&self, length: usize) -> bool {
        self.asked.borrow_mut().push(Asked::CmdLimit(length));
        self.cmd_limit
    }

    fn hard_cap(&self, length: usize) {
        self.asked.borrow_mut().push(Asked::HardCap(length));
    }
}

/// One Scope as a run is handed it, typed `REG_EXPAND_SZ` — the type an Absent
/// Scope's Working Copy carries, and the one a test only names when it is the
/// point.
fn input(scope: Scope, key: &TestKey, entries: &[&str], last_read: RawValue) -> ScopeInput {
    ScopeInput {
        scope,
        key: key.key(),
        entries: entries.iter().map(|entry| entry.to_string()).collect(),
        value_type: ValueType::RegExpandSz,
        last_read,
    }
}

/// The Scope this run is not applying: empty, and never read.
fn idle(scope: Scope, key: &TestKey) -> ScopeInput {
    input(scope, key, &[], RawValue::Absent)
}

/// A run over `order`, with everything a test usually leaves alone filled in:
/// no log, a fixed clock, and a budget wide enough that rotation is invisible
/// until a test makes it the subject.
fn run<'a>(world: &'a World, scopes: [ScopeInput; 2], order: &'a [Scope]) -> ApplyRun<'a> {
    ApplyRun {
        scopes,
        order,
        data_dir: world.data_dir(),
        log_path: None,
        at: at(7),
        max_backups: 50,
    }
}

/// What one Scope's outcome says, as a word — so an assertion reads as the
/// sentence it is checking rather than as a `matches!`.
fn outcome_word(outcome: &ScopeOutcome) -> &'static str {
    match outcome {
        ScopeOutcome::Applied { .. } => "applied",
        ScopeOutcome::Refreshed { .. } => "refreshed",
        ScopeOutcome::Cancelled => "cancelled",
        ScopeOutcome::Failed(_) => "failed",
    }
}

fn words(outcome: &apply::Outcome) -> Vec<(Scope, &'static str)> {
    outcome
        .scopes
        .iter()
        .map(|(scope, done)| (*scope, outcome_word(done)))
        .collect()
}

fn log_lines(outcome: &apply::Outcome) -> Vec<String> {
    outcome
        .records
        .iter()
        .map(|record| pathmaster_core::logfmt::line(&at(7), record))
        .collect()
}

/// An entry long enough to push the merged length past a threshold on its own.
fn long_entry(chars: usize) -> String {
    format!(r"C:\{}", "x".repeat(chars - 3))
}

// ---- The healthy path ----

#[test]
fn an_apply_writes_the_working_copy_and_hands_back_what_it_now_holds() {
    let world = World::new("healthy");
    let last_read = world.user.plant(ValueType::RegExpandSz, r"C:\old");
    let ask = Scripted::new();

    let outcome = apply::apply(
        run(
            &world,
            [
                input(Scope::User, &world.user, &[r"C:\a", r"C:\b"], last_read),
                idle(Scope::System, &world.system),
            ],
            &[Scope::User],
        ),
        &Env,
        &ask,
    );

    assert_eq!(words(&outcome), vec![(Scope::User, "applied")]);
    assert!(outcome.completed());
    assert_eq!(
        world.user.key().read().unwrap().decode(),
        pathmaster_core::session::ScopeValue::Present {
            value_type: ValueType::RegExpandSz,
            raw: r"C:\a;C:\b".to_string(),
        },
    );
    // The value handed back is what the *next* run compares against, so it has
    // to be byte-for-byte what the registry now holds (spec §4).
    let ScopeOutcome::Applied { stored } = &outcome.scopes[0].1 else {
        panic!("applied");
    };
    assert_eq!(stored, &world.user.key().read().unwrap());
    // Within every threshold and unchanged since the last read: nothing to ask.
    assert_eq!(ask.asked(), Vec::new());
}

#[test]
fn an_apply_logs_one_audit_line_carrying_facts_and_no_path_text() {
    let world = World::new("audit");
    let last_read = world.user.plant(ValueType::RegSz, r"C:\old");
    let mut scopes = [
        input(Scope::User, &world.user, &[r"C:\a", r"C:\bb"], last_read),
        idle(Scope::System, &world.system),
    ];
    scopes[0].value_type = ValueType::RegSz;

    let outcome = apply::apply(run(&world, scopes, &[Scope::User]), &Env, &Scripted::new());

    // `C:\a;C:\bb` is ten UTF-16 code units — the unit Windows stores in.
    assert_eq!(
        log_lines(&outcome),
        vec!["2026-08-21T14:32:07+03:00 INFO  apply: \
             User scope written, 2 entries, 10 chars, REG_SZ\n"
            .to_string()],
    );
}

#[test]
fn an_apply_backs_up_the_value_it_re_read_and_not_the_one_it_is_writing() {
    let world = World::new("backup-subject");
    let last_read = world.user.plant(ValueType::RegSz, r"C:\before;C:\second");

    apply::apply(
        run(
            &world,
            [
                input(Scope::User, &world.user, &[r"C:\after"], last_read),
                idle(Scope::System, &world.system),
            ],
            &[Scope::User],
        ),
        &Env,
        &Scripted::new(),
    );

    // Decoded, not raw: a Snapshot records Entries and the Value Type they
    // were stored under (ADR-0006). And it records the value the registry
    // held, which is the only one that would otherwise be gone.
    assert_eq!(
        world.only_snapshot(),
        Snapshot {
            timestamp: "2026-08-21T14-32-07".to_string(),
            scope: Scope::User,
            captured: Captured::Present {
                value_type: ValueType::RegSz,
                entries: vec![r"C:\before".to_string(), r"C:\second".to_string()],
            },
        },
    );
}

#[test]
fn a_first_apply_over_an_absent_scope_creates_the_value_and_backs_up_absent() {
    // An Absent Scope's Working Copy is typed `REG_EXPAND_SZ` at load, so the
    // first Apply creates it as that (spec §4). Its Snapshot has no Value Type
    // at all, because an Absent Scope has no value to have one (ADR-0006).
    let world = World::new("absent");

    apply::apply(
        run(
            &world,
            [
                input(Scope::User, &world.user, &[r"C:\new"], RawValue::Absent),
                idle(Scope::System, &world.system),
            ],
            &[Scope::User],
        ),
        &Env,
        &Scripted::new(),
    );

    assert_eq!(world.only_snapshot().captured, Captured::Absent);
    assert_eq!(
        world.user.key().read().unwrap(),
        RawValue::written(ValueType::RegExpandSz, r"C:\new"),
    );
}

#[test]
fn zero_entries_over_a_present_scope_writes_an_empty_value_and_never_deletes_it() {
    let world = World::new("empty");
    let last_read = world.user.plant(ValueType::RegExpandSz, r"C:\going");

    apply::apply(
        run(
            &world,
            [
                input(Scope::User, &world.user, &[], last_read),
                idle(Scope::System, &world.system),
            ],
            &[Scope::User],
        ),
        &Env,
        &Scripted::new(),
    );

    // Present with a lone NUL — emphatically not Absent (spec §4).
    assert_eq!(
        world.user.key().read().unwrap(),
        RawValue::Present {
            value_type: ValueType::RegExpandSz,
            bytes: vec![0, 0],
        },
    );
}

// ---- The external-change dialog (spec §5, FR-apply) ----

#[test]
fn overwrite_proceeds_and_backs_up_what_was_found_not_what_the_session_remembered() {
    // The sharpest reason the backup is of the re-read value: after an
    // external edit, the Baseline is precisely *not* what is about to be
    // overwritten, so backing it up would preserve a value nobody has.
    let world = World::new("overwrite");
    let remembered = world.user.plant(ValueType::RegExpandSz, r"C:\remembered");
    world.user.plant(ValueType::RegExpandSz, r"C:\someone-else");
    let ask = Scripted::answering(ExternalChange::Overwrite);

    let outcome = apply::apply(
        run(
            &world,
            [
                input(Scope::User, &world.user, &[r"C:\mine"], remembered),
                idle(Scope::System, &world.system),
            ],
            &[Scope::User],
        ),
        &Env,
        &ask,
    );

    assert_eq!(ask.asked(), vec![Asked::ExternalChange(Scope::User)]);
    assert_eq!(words(&outcome), vec![(Scope::User, "applied")]);
    assert_eq!(
        world.only_snapshot().captured,
        Captured::Present {
            value_type: ValueType::RegExpandSz,
            entries: vec![r"C:\someone-else".to_string()],
        },
    );
    assert_eq!(
        world.user.key().read().unwrap(),
        RawValue::written(ValueType::RegExpandSz, r"C:\mine"),
    );
}

#[test]
fn refresh_and_discard_writes_nothing_takes_no_backup_and_still_completes() {
    // A Scope a run completes is not necessarily one it applied (`CONTEXT.md`):
    // this answer adopts the value that was just read, so the run carries on
    // to the next Scope.
    let world = World::new("refresh");
    let remembered = world.user.plant(ValueType::RegExpandSz, r"C:\remembered");
    let found = world.user.plant(ValueType::RegExpandSz, r"C:\someone-else");

    let outcome = apply::apply(
        run(
            &world,
            [
                input(Scope::User, &world.user, &[r"C:\mine"], remembered),
                idle(Scope::System, &world.system),
            ],
            &[Scope::User],
        ),
        &Env,
        &Scripted::answering(ExternalChange::RefreshAndDiscard),
    );

    assert_eq!(words(&outcome), vec![(Scope::User, "refreshed")]);
    assert!(outcome.completed());
    let ScopeOutcome::Refreshed { found: handed } = &outcome.scopes[0].1 else {
        panic!("refreshed");
    };
    assert_eq!(handed, &found, "the window adopts what the run just read");
    assert_eq!(world.user.key().read().unwrap(), found, "nothing written");
    assert!(
        world.snapshots().is_empty(),
        "no backup for a write that never happened"
    );
    assert!(outcome.records.is_empty(), "nothing to audit");
}

#[test]
fn cancelling_the_external_change_dialog_writes_nothing_and_stops_the_run() {
    let world = World::new("external-cancel");
    let remembered = world.user.plant(ValueType::RegExpandSz, r"C:\remembered");
    let found = world.user.plant(ValueType::RegExpandSz, r"C:\someone-else");

    let outcome = apply::apply(
        run(
            &world,
            [
                input(Scope::User, &world.user, &[r"C:\mine"], remembered),
                input(Scope::System, &world.system, &[r"C:\sys"], RawValue::Absent),
            ],
            &[Scope::User, Scope::System],
        ),
        &Env,
        &Scripted::answering(ExternalChange::Cancel),
    );

    // A user's Cancel stops the run exactly as a failure does: the second
    // Scope is never reached (spec §5, ADR-0008).
    assert_eq!(words(&outcome), vec![(Scope::User, "cancelled")]);
    assert!(!outcome.completed());
    assert_eq!(world.user.key().read().unwrap(), found);
    assert_eq!(world.system.key().read().unwrap(), RawValue::Absent);
    assert!(world.snapshots().is_empty());
}

#[test]
fn a_value_that_has_not_moved_raises_no_dialog_at_all() {
    // Detection is a comparison of `(vtype, bytes)`, so an identical re-read is
    // silence. Nothing watches and nothing polls — this is the only moment the
    // question is ever asked (spec §4, §5).
    let world = World::new("unmoved");
    let last_read = world.user.plant(ValueType::RegExpandSz, r"C:\same");
    let ask = Scripted::answering(ExternalChange::Cancel);

    let outcome = apply::apply(
        run(
            &world,
            [
                input(
                    Scope::User,
                    &world.user,
                    &[r"C:\same", r"C:\more"],
                    last_read,
                ),
                idle(Scope::System, &world.system),
            ],
            &[Scope::User],
        ),
        &Env,
        &ask,
    );

    assert_eq!(ask.asked(), Vec::new());
    assert_eq!(words(&outcome), vec![(Scope::User, "applied")]);
}

#[test]
fn a_value_type_change_alone_is_an_external_change() {
    // The comparison is `(vtype, bytes)` and not the bytes: a Scope someone
    // else converted to `REG_SZ` holds the same text and a different value.
    let world = World::new("vtype-moved");
    let remembered = world.user.plant(ValueType::RegExpandSz, r"C:\same");
    world.user.plant(ValueType::RegSz, r"C:\same");
    let ask = Scripted::answering(ExternalChange::Cancel);

    apply::apply(
        run(
            &world,
            [
                input(Scope::User, &world.user, &[r"C:\same"], remembered),
                idle(Scope::System, &world.system),
            ],
            &[Scope::User],
        ),
        &Env,
        &ask,
    );

    assert_eq!(ask.asked(), vec![Asked::ExternalChange(Scope::User)]);
}

// ---- The failure taxonomy (spec §9) ----

/// A key whose name is longer than the registry allows, so every call against
/// it fails with a real OS error rather than a simulated one.
fn unreachable_key() -> ScopeKey {
    ScopeKey::at(
        Hive::CurrentUser,
        format!(r"{TEST_ROOT}\{}", "x".repeat(300)),
        "Path",
    )
}

/// A key that reads (the value is simply Absent) but cannot be written: the
/// value name is past the registry's own limit.
fn read_only_key(key: &TestKey) -> ScopeKey {
    winreg::RegKey::predef(winreg::enums::HKEY_CURRENT_USER)
        .create_subkey(&key.path)
        .expect("the key itself");
    ScopeKey::at(Hive::CurrentUser, &key.path, "V".repeat(20_000))
}

#[test]
fn a_re_read_that_fails_writes_nothing_takes_no_backup_and_names_the_registry() {
    // §9's fifth row. Proceeding without the comparison was rejected outright:
    // it is exactly the case where an external change is overwritten with no
    // dialog, which is the hazard the whole order exists for.
    let world = World::new("reread-fails");
    let mut scopes = [
        input(Scope::User, &world.user, &[r"C:\a"], RawValue::Absent),
        idle(Scope::System, &world.system),
    ];
    scopes[0].key = unreachable_key();

    let outcome = apply::apply(run(&world, scopes, &[Scope::User]), &Env, &Scripted::new());

    assert_eq!(words(&outcome), vec![(Scope::User, "failed")]);
    let ScopeOutcome::Failed(failure) = &outcome.scopes[0].1 else {
        panic!("failed");
    };
    // The registry-write row's own cause: nothing was written either way.
    assert_eq!(failure.catalogue_msgid(), msgids::APPLY_FAILED_REGISTRY);
    assert!(world.snapshots().is_empty());
    assert_eq!(
        log_lines(&outcome),
        vec!["2026-08-21T14:32:07+03:00 ERROR apply: \
             User scope not applied, re-read failed (os error 87)\n"
            .to_string()],
    );
}

#[test]
fn a_backup_that_cannot_be_written_stops_before_the_registry_is_touched() {
    // The order's whole point: the value the user is about to lose is on disk
    // before anything overwrites it, so a run that cannot take the backup does
    // not write at all (spec §8, §9).
    let world = World::new("backup-fails");
    let last_read = world.user.plant(ValueType::RegExpandSz, r"C:\untouched");
    // A file squatting on `data\backups\`: `create_dir_all` cannot make the
    // directory, so the Snapshot cannot be written.
    fs::write(world.backups(), b"not a directory").unwrap();

    let outcome = apply::apply(
        run(
            &world,
            [
                input(Scope::User, &world.user, &[r"C:\new"], last_read.clone()),
                idle(Scope::System, &world.system),
            ],
            &[Scope::User],
        ),
        &Env,
        &Scripted::new(),
    );

    let ScopeOutcome::Failed(failure) = &outcome.scopes[0].1 else {
        panic!("failed");
    };
    assert_eq!(failure.catalogue_msgid(), msgids::APPLY_FAILED_BACKUP);
    assert_eq!(
        world.user.key().read().unwrap(),
        last_read,
        "the registry must be exactly as the run found it"
    );
    assert_eq!(
        outcome.records.len(),
        1,
        "one record, and it is the failure"
    );
    assert!(log_lines(&outcome)[0].contains("backup failed"));
}

#[test]
fn a_registry_write_that_fails_leaves_the_backup_it_had_already_taken() {
    // The Snapshot is not rolled back, and that is deliberate: it is a true
    // record of what the Scope held, and the one thing a user whose Apply just
    // failed might want.
    let world = World::new("write-fails");
    let mut scopes = [
        input(Scope::User, &world.user, &[r"C:\a"], RawValue::Absent),
        idle(Scope::System, &world.system),
    ];
    scopes[0].key = read_only_key(&world.user);

    let outcome = apply::apply(run(&world, scopes, &[Scope::User]), &Env, &Scripted::new());

    let ScopeOutcome::Failed(failure) = &outcome.scopes[0].1 else {
        panic!("failed");
    };
    assert_eq!(failure.catalogue_msgid(), msgids::APPLY_FAILED_REGISTRY);
    assert_eq!(world.only_snapshot().captured, Captured::Absent);
    assert_eq!(
        log_lines(&outcome),
        vec!["2026-08-21T14:32:07+03:00 ERROR apply: \
             User scope not applied, registry write failed (os error 87)\n"
            .to_string()],
    );
}

#[test]
fn access_denied_speaks_its_own_cause_and_everything_else_the_general_one() {
    // §9's example text is "Apply failed — access denied.", which is the
    // unelevated System Scope and by far the likeliest failure there is. Every
    // other way the registry can refuse takes the general phrase, because a
    // spoken sentence cannot carry an OS error code.
    let denied = Failure::Write(RegistryError::Io(io::Error::from_raw_os_error(5)));
    assert_eq!(denied.catalogue_msgid(), msgids::APPLY_FAILED_ACCESS_DENIED);
    let locked = Failure::Write(RegistryError::Io(io::Error::from_raw_os_error(32)));
    assert_eq!(locked.catalogue_msgid(), msgids::APPLY_FAILED_REGISTRY);
    let foreign = Failure::ReRead(RegistryError::UnsupportedType(3));
    assert_eq!(foreign.catalogue_msgid(), msgids::APPLY_FAILED_REGISTRY);
    // A failed backup says nothing about the registry at all — it says the one
    // thing that matters, which is that no change was made.
    let backup = Failure::Snapshot(io::Error::from_raw_os_error(5));
    assert_eq!(backup.catalogue_msgid(), msgids::APPLY_FAILED_BACKUP);
}

// ---- Rotation (spec §8, FR-backup-rotation) ----

#[test]
fn rotation_runs_after_the_write_and_over_the_scope_that_was_written() {
    let world = World::new("rotation");
    let last_read = world.user.plant(ValueType::RegExpandSz, r"C:\v1");
    let scopes = || {
        [
            input(Scope::User, &world.user, &[r"C:\v2"], last_read.clone()),
            idle(Scope::System, &world.system),
        ]
    };
    // Two earlier Applies, each a second apart, so the budget of two is
    // already full when the third arrives.
    for second in [1, 2] {
        let mut this = run(&world, scopes(), &[Scope::User]);
        this.at = at(second);
        this.max_backups = 2;
        apply::apply(this, &Env, &Scripted::new());
    }
    assert_eq!(world.snapshots().len(), 2);

    let mut third = run(&world, scopes(), &[Scope::User]);
    third.at = at(3);
    third.max_backups = 2;
    apply::apply(third, &Env, &Scripted::new());

    let kept: Vec<String> = world
        .snapshots()
        .iter()
        .map(|name| name.timestamp().to_owned())
        .collect();
    assert_eq!(
        kept,
        vec!["2026-08-21T14-32-02", "2026-08-21T14-32-03"],
        "the oldest of this Scope goes, and the write's own Snapshot stays"
    );
}

#[test]
fn the_rotation_after_an_apply_never_deletes_the_backup_that_apply_just_took() {
    // Every Apply in this test happens inside one second, so the collision
    // suffix *is* the age. The rule that a suffix rotation frees is never
    // reissued is what stops the third Apply's own Snapshot — which would
    // otherwise be handed the freed plain name, the oldest of the second — from
    // being the file its own rotation deletes (spec §8).
    let world = World::new("same-second");
    let last_read = world.user.plant(ValueType::RegExpandSz, r"C:\v1");
    let mut newest = String::new();
    for _ in 0..3 {
        let mut this = run(
            &world,
            [
                input(Scope::User, &world.user, &[r"C:\v2"], last_read.clone()),
                idle(Scope::System, &world.system),
            ],
            &[Scope::User],
        );
        this.max_backups = 1;
        let outcome = apply::apply(this, &Env, &Scripted::new());
        assert_eq!(words(&outcome), vec![(Scope::User, "applied")]);
        let listing = world.snapshots();
        assert_eq!(listing.len(), 1, "the budget is one: {listing:?}");
        let kept = listing[0].file_name().to_owned();
        assert_ne!(kept, newest, "each Apply's own Snapshot is the one kept");
        newest = kept;
    }
    assert_eq!(newest, "2026-08-21T14-32-07-User-2.json");
}

// ---- The over-length gates (spec §7, FR-diag-overlength) ----

#[test]
fn a_length_past_the_cmd_limit_is_a_warning_the_user_may_walk_past() {
    let world = World::new("cmd-limit");
    let ask = Scripted::new();

    let outcome = apply::apply(
        run(
            &world,
            [
                input(
                    Scope::User,
                    &world.user,
                    &[&long_entry(9_000)],
                    RawValue::Absent,
                ),
                idle(Scope::System, &world.system),
            ],
            &[Scope::User],
        ),
        &Env,
        &ask,
    );

    // The number in the dialog is the merged length this Apply would leave
    // behind: 9,000 for User, plus the separator, plus an empty System.
    assert_eq!(ask.asked(), vec![Asked::CmdLimit(9_001)]);
    assert_eq!(words(&outcome), vec![(Scope::User, "applied")]);
}

#[test]
fn a_cmd_limit_warning_the_user_refuses_writes_nothing() {
    let world = World::new("cmd-limit-refused");

    let outcome = apply::apply(
        run(
            &world,
            [
                input(
                    Scope::User,
                    &world.user,
                    &[&long_entry(9_000)],
                    RawValue::Absent,
                ),
                idle(Scope::System, &world.system),
            ],
            &[Scope::User],
        ),
        &Env,
        &Scripted::refusing_the_cmd_limit(),
    );

    assert_eq!(words(&outcome), vec![(Scope::User, "cancelled")]);
    assert!(!outcome.completed());
    assert_eq!(world.user.key().read().unwrap(), RawValue::Absent);
    assert!(world.snapshots().is_empty());
}

#[test]
fn a_length_at_the_hard_cap_is_told_and_never_asked() {
    // The gate has no proceed button, and the port has no answer to give: the
    // signature is the rule (spec §7).
    let world = World::new("hard-cap");
    let ask = Scripted::new();

    let outcome = apply::apply(
        run(
            &world,
            [
                input(
                    Scope::User,
                    &world.user,
                    &[&long_entry(40_000)],
                    RawValue::Absent,
                ),
                idle(Scope::System, &world.system),
            ],
            &[Scope::User],
        ),
        &Env,
        &ask,
    );

    assert_eq!(ask.asked(), vec![Asked::HardCap(40_001)]);
    assert_eq!(words(&outcome), vec![(Scope::User, "cancelled")]);
    assert_eq!(world.user.key().read().unwrap(), RawValue::Absent);
    assert!(world.snapshots().is_empty());
}

#[test]
fn the_gate_measures_both_working_copies_however_few_scopes_are_applied() {
    // The merged length is a fact about the pair. A User Apply that adds one
    // character to an already-enormous System Scope is still the Apply that
    // takes the machine past the limit, and the dialog says so.
    let world = World::new("gate-both");
    let ask = Scripted::new();

    apply::apply(
        run(
            &world,
            [
                input(Scope::User, &world.user, &[r"C:\a"], RawValue::Absent),
                input(
                    Scope::System,
                    &world.system,
                    &[&long_entry(9_000)],
                    RawValue::Absent,
                ),
            ],
            &[Scope::User],
        ),
        &Env,
        &ask,
    );

    assert_eq!(ask.asked(), vec![Asked::CmdLimit(9_005)]);
}

// ---- A run covers Scopes, not a Scope (ADR-0008) ----

#[test]
fn a_run_takes_the_scopes_in_the_order_it_is_given() {
    let world = World::new("order");

    let outcome = apply::apply(
        run(
            &world,
            [
                input(Scope::User, &world.user, &[r"C:\u"], RawValue::Absent),
                input(Scope::System, &world.system, &[r"C:\s"], RawValue::Absent),
            ],
            &[Scope::User, Scope::System],
        ),
        &Env,
        &Scripted::new(),
    );

    assert_eq!(
        words(&outcome),
        vec![(Scope::User, "applied"), (Scope::System, "applied")],
    );
    assert!(outcome.completed());
    assert_eq!(
        world.snapshots().len(),
        2,
        "one Snapshot per Scope written, each of its own Scope's value"
    );
    assert_eq!(outcome.records.len(), 2, "one audit line per Scope written");
}

#[test]
fn a_run_stops_at_the_first_scope_that_does_not_complete() {
    let world = World::new("stop");
    let mut scopes = [
        input(Scope::User, &world.user, &[r"C:\u"], RawValue::Absent),
        input(Scope::System, &world.system, &[r"C:\s"], RawValue::Absent),
    ];
    scopes[0].key = unreachable_key();

    let outcome = apply::apply(
        run(&world, scopes, &[Scope::User, Scope::System]),
        &Env,
        &Scripted::new(),
    );

    assert_eq!(words(&outcome), vec![(Scope::User, "failed")]);
    assert_eq!(
        world.system.key().read().unwrap(),
        RawValue::Absent,
        "the Scope after the failure is never reached"
    );
}

#[test]
fn a_run_over_no_scopes_does_nothing_and_asks_nothing() {
    let world = World::new("nothing");
    let ask = Scripted::new();

    let outcome = apply::apply(
        run(
            &world,
            [
                input(
                    Scope::User,
                    &world.user,
                    &[&long_entry(40_000)],
                    RawValue::Absent,
                ),
                idle(Scope::System, &world.system),
            ],
            &[],
        ),
        &Env,
        &ask,
    );

    assert_eq!(ask.asked(), Vec::new(), "not even the gate");
    assert!(outcome.scopes.is_empty());
    assert!(outcome.completed());
}
