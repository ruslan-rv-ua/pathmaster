//! The diagnostic pass's shell: the two adapters that answer the rulebook's
//! questions about this machine, and the worker thread that runs it (spec §7,
//! FR-diag-async, ticket impl-12).
//!
//! The adapters are measured against the real filesystem and the real process
//! environment, for the same reason the registry tests use the live registry:
//! every hazard they exist for — a quoted path Win32 rejects outright, a
//! forward-slashed one it resolves anyway — is real API behaviour, and a mock
//! would only repeat what the adapter already assumes.
//!
//! One branch is deliberately **not** tested here: `Existence::AccessDenied`.
//! Measured, this machine will not produce it — `GetFileAttributesW` needs
//! only `FILE_READ_ATTRIBUTES`, which rides the parent's traverse right, so a
//! deny-ACL'd directory and `C:\System Volume Information` both read their
//! attributes back on an ordinary account. A test that denied read and then
//! asserted the answer would pass identically with the deny removed, which is
//! no test at all. The rule that branch feeds — access denied is never
//! Missing — is held in `core/tests/diagnostics.rs`, where a fake filesystem
//! can simply say so.
//!
//! The worker is driven through a filesystem the test can hold open mid-pass,
//! so "a pass the screen has outrun never reaches the UI" is asserted rather
//! than timed.

#![cfg(windows)]

use std::fs;
use std::path::Path;
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::Mutex;
use std::time::{Duration, Instant};

use pathmaster_core::diagnostics::{Diagnosis, Existence, Filesystem, Issue, RootKind};
use pathmaster_core::normalize::Environment;
use pathmaster_core::session::Scope;
use pathmaster_platform::diagnostics::{LocalFilesystem, ProcessEnvironment, Worker};

// ---- The process environment, as expansion reads it ----

#[test]
fn the_process_environment_answers_whatever_the_case() {
    // `GetEnvironmentVariableW` ignores case, so `%systemroot%` and
    // `%SystemRoot%` must name one variable (FR-diag-normalise).
    let env = ProcessEnvironment;
    let value = env
        .lookup("SystemRoot")
        .expect("SystemRoot is defined on every Windows run");
    assert_eq!(env.lookup("systemroot").as_deref(), Some(value.as_str()));
    assert_eq!(env.lookup("SYSTEMROOT").as_deref(), Some(value.as_str()));
}

#[test]
fn an_undefined_name_is_none_not_an_empty_value() {
    // The rules turn `None` into "left literal, and reported as unresolved";
    // an empty string would silently expand the reference away.
    assert_eq!(
        ProcessEnvironment.lookup("PATHMASTER_NO_SUCH_VARIABLE"),
        None
    );
}

// ---- The filesystem: root classification, then the probe ----

#[test]
fn a_drive_letter_is_local_and_a_unc_prefix_is_network() {
    let fs = LocalFilesystem;
    assert_eq!(fs.root_kind(r"C:\Windows"), RootKind::Local);
    assert_eq!(fs.root_kind(r"C:/Windows"), RootKind::Local);
    // Never probed in v0.1.0: a dead UNC blocks 20-60 s uncancellably.
    assert_eq!(fs.root_kind(r"\\server\share\bin"), RootKind::Network);
    assert_eq!(fs.root_kind("//server/share/bin"), RootKind::Network);
}

#[test]
fn a_device_namespace_path_is_treated_as_network_and_left_alone() {
    // `\\?\UNC\server\share` is a UNC path wearing a prefix, and telling the
    // two apart is exactly the round trip the network rule exists to avoid.
    // Everything under `\\` is therefore left unprobed — a false negative on a
    // spelling almost nothing uses, and never a 60-second window.
    assert_eq!(
        LocalFilesystem.root_kind(r"\\?\UNC\server\share"),
        RootKind::Network
    );
    assert_eq!(
        LocalFilesystem.root_kind(r"\\?\C:\Windows"),
        RootKind::Network
    );
}

#[test]
fn a_path_with_no_root_at_all_is_local() {
    // An Entry whose leading `%VAR%` this run does not define reaches the
    // probe as literal text (spec §7, D10). It has no root to classify, and
    // it must be probed — the probe is what makes it flag Missing.
    assert_eq!(LocalFilesystem.root_kind(r"%NOPE%\bin"), RootKind::Local);
}

#[test]
fn an_existing_directory_is_the_only_healthy_answer() {
    let dir = tempfile::tempdir().unwrap();
    assert_eq!(probe(dir.path()), Existence::Directory);
}

#[test]
fn a_trailing_separator_and_a_forward_slash_name_the_same_directory() {
    // Measured (ticket impl-09): Win32 resolves `C:/Windows` exactly as
    // `C:\Windows`, which is why the probe reads the expanded text verbatim
    // and slash direction belongs to the comparison key alone.
    let fs = LocalFilesystem;
    let windows = ProcessEnvironment
        .lookup("SystemRoot")
        .expect("SystemRoot is defined on every Windows run");
    assert_eq!(fs.probe(&windows), Existence::Directory);
    assert_eq!(fs.probe(&format!("{windows}\\")), Existence::Directory);
    assert_eq!(fs.probe(&windows.replace('\\', "/")), Existence::Directory);
}

#[test]
fn an_existing_file_is_not_a_directory() {
    // Inert in a `PATH`: the search appends `\name.exe`, so a file Entry finds
    // nothing — and the rules flag it exactly as they flag a missing one.
    let dir = tempfile::tempdir().unwrap();
    let file = dir.path().join("notes.txt");
    fs::write(&file, b"").unwrap();
    assert_eq!(probe(&file), Existence::File);
}

#[test]
fn nothing_of_that_name_is_not_found() {
    let dir = tempfile::tempdir().unwrap();
    assert_eq!(
        probe(&dir.path().join("no-such-thing")),
        Existence::NotFound
    );
}

#[test]
fn a_quoted_path_is_not_found_rather_than_stripped_by_win32() {
    // Measured (ticket impl-09): `GetFileAttributesW` fails `"C:\Windows"`
    // with `ERROR_INVALID_NAME`. The rules strip the quotes before probing —
    // this is what would happen if they stopped.
    let dir = tempfile::tempdir().unwrap();
    let quoted = format!("\"{}\"", dir.path().display());
    assert_eq!(LocalFilesystem.probe(&quoted), Existence::NotFound);
}

fn probe(path: &Path) -> Existence {
    LocalFilesystem.probe(&path.display().to_string())
}

// ---- The worker thread ----

#[test]
fn a_pass_over_this_machine_finds_what_is_really_there() {
    let dir = tempfile::tempdir().unwrap();
    let present = dir.path().display().to_string();
    let absent = dir.path().join("no-such-thing").display().to_string();

    let mut worker = Worker::spawn();
    worker.request(Vec::new(), vec![present, absent]);
    let diagnosis = settle(&mut worker);

    assert_eq!(diagnosis.scope(Scope::User).issues(0), &[] as &[Issue]);
    assert_eq!(diagnosis.scope(Scope::User).issues(1), &[Issue::Missing]);
}

#[test]
fn the_pass_expands_variables_against_the_process_environment() {
    // Unexpanded, `%SystemRoot%` is literal text that names no directory and
    // would flag Missing — so a healthy verdict is expansion, measured.
    let mut worker = Worker::spawn();
    worker.request(vec!["%SystemRoot%".to_string()], Vec::new());
    let diagnosis = settle(&mut worker);
    assert_eq!(diagnosis.scope(Scope::System).issues(0), &[] as &[Issue]);
}

#[test]
fn nothing_is_outstanding_until_a_pass_is_asked_for() {
    // The Timer runs only while a pass is outstanding, so this is what keeps
    // it stopped at rest.
    let mut worker = Worker::spawn();
    assert!(!worker.outstanding());
    assert!(worker.take().is_none());

    worker.request(Vec::new(), Vec::new());
    assert!(worker.outstanding());
    settle(&mut worker);
    assert!(!worker.outstanding());
}

#[test]
fn a_pass_the_working_copies_have_outrun_never_reaches_the_ui() {
    // The findings of a pass are read against the Entries it ran over; one
    // that has been overtaken describes a screen that no longer exists.
    let (fs, gate) = Gated::new();
    let mut worker = Worker::spawn_over(Box::new(Undefined), Box::new(fs));

    worker.request(Vec::new(), vec![entry("one")]);
    assert_eq!(gate.next_probe(), entry("one"), "the first pass is running");
    assert!(worker.take().is_none(), "and has not finished");

    // A second pass is asked for while the first is still inside its probe.
    worker.request(Vec::new(), vec![entry("two"), entry("three")]);
    gate.release();

    // The worker is single-threaded: seeing the second pass probe proves the
    // first has already finished and replied.
    assert_eq!(gate.next_probe(), entry("two"));
    gate.release();
    assert_eq!(gate.next_probe(), entry("three"));
    gate.release();

    let diagnosis = settle(&mut worker);
    assert_eq!(
        diagnosis.scope(Scope::User).len(),
        2,
        "the overtaken pass was dropped, not shown"
    );
    assert!(!worker.outstanding());
}

#[test]
fn a_burst_of_edits_is_one_pass_over_the_newest_working_copies() {
    // Every keystroke-fast operation asks for a pass; running each of them in
    // turn would spend the budget on states the user has already left.
    let (fs, gate) = Gated::new();
    let mut worker = Worker::spawn_over(Box::new(Undefined), Box::new(fs));

    worker.request(Vec::new(), vec![entry("one")]);
    assert_eq!(gate.next_probe(), entry("one"));
    worker.request(Vec::new(), vec![entry("skipped")]);
    worker.request(Vec::new(), vec![entry("three"), entry("four")]);
    gate.release();

    // The middle request is never run: the next probe is the last one's.
    assert_eq!(gate.next_probe(), entry("three"));
    gate.release();
    assert_eq!(gate.next_probe(), entry("four"));
    gate.release();

    assert_eq!(settle(&mut worker).scope(Scope::User).len(), 2);
}

/// An Entry that is fully qualified and local, so it reaches the probe.
fn entry(name: &str) -> String {
    format!(r"C:\{name}")
}

/// Polls the worker the way the wx Timer does, until the pass it is waiting
/// for lands.
fn settle(worker: &mut Worker) -> Diagnosis {
    for _ in 0..400 {
        if let Some(diagnosis) = worker.take() {
            return diagnosis;
        }
        std::thread::sleep(Duration::from_millis(5));
    }
    panic!("the pass never landed");
}

/// A filesystem that reports each probe and then waits to be let through, so a
/// test can hold a pass inside the worker while it asks for another.
struct Gated {
    probing: Sender<String>,
    proceed: Mutex<Receiver<()>>,
}

/// The test's end of a [`Gated`] filesystem.
struct Gate {
    probing: Receiver<String>,
    proceed: Sender<()>,
}

impl Gated {
    fn new() -> (Gated, Gate) {
        let (probing, probed) = mpsc::channel();
        let (proceed, allowed) = mpsc::channel();
        (
            Gated {
                probing,
                proceed: Mutex::new(allowed),
            },
            Gate {
                probing: probed,
                proceed,
            },
        )
    }
}

impl Gate {
    /// The path the worker is waiting inside — blocking, so a test that races
    /// the worker hangs its own assertion rather than passing by luck.
    fn next_probe(&self) -> String {
        self.probing
            .recv_timeout(Duration::from_secs(5))
            .expect("the worker reached a probe")
    }

    fn release(&self) {
        self.proceed.send(()).expect("the worker is still probing");
    }
}

impl Filesystem for Gated {
    fn root_kind(&self, _path: &str) -> RootKind {
        RootKind::Local
    }

    fn probe(&self, path: &str) -> Existence {
        self.probing.send(path.to_string()).ok();
        self.proceed.lock().unwrap().recv().ok();
        Existence::Directory
    }
}

/// An environment that defines nothing, so a gated pass's Entries reach the
/// probe as the text the test wrote.
struct Undefined;

impl Environment for Undefined {
    fn lookup(&self, _name: &str) -> Option<String> {
        None
    }
}

#[test]
fn a_full_pass_over_two_hundred_entries_is_well_inside_the_budget() {
    // Spec §7's budget is one second for 200 Entries. The bound here is not a
    // performance gate — the measured pass is two orders of magnitude under it
    // (see the ticket) — it is a **network gate**: the one way this pass can
    // take seconds is by probing a root it was told never to touch, and a dead
    // UNC blocks 20-60 s on its own. Anything that reintroduces that fails
    // here rather than in front of the user.
    let dir = tempfile::tempdir().unwrap();
    let entries: Vec<String> = (0..200)
        .map(|i| dir.path().join(format!("entry-{i}")).display().to_string())
        .collect();

    let mut worker = Worker::spawn();
    let started = Instant::now();
    worker.request(Vec::new(), entries);
    let diagnosis = settle(&mut worker);
    let elapsed = started.elapsed();

    assert_eq!(diagnosis.scope(Scope::User).len(), 200);
    assert!(
        elapsed < Duration::from_secs(1),
        "a 200-entry pass took {elapsed:?}, past spec §7's one-second budget"
    );
}
