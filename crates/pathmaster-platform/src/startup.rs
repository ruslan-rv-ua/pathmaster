//! Everything a **Run** is, decided in one place (spec §11, §3, §9, §13, §14;
//! ADR-0010).
//!
//! A Run is one execution of the application, and a handful of things are
//! settled at its start and never again: where its data lives and whether it
//! can be written, whether it is elevated, what language it speaks, whether it
//! has a log at all, and which Scopes it may write. Every one of those used to
//! be decided in `main`, between calls that each had tests of their own — and
//! the glue between them, which had none, is exactly **seven rules**:
//!
//! 1. Read-only Data is a Run without a log — the log lives in the Data
//!    Directory, and an unopenable log stays a Run without a log rather than
//!    becoming Read-only Data (spec §14).
//! 2. The panic hook installs only where there is a log path to install
//!    against.
//! 3. The startup record precedes the settings records: it names the build,
//!    and until it does nothing under it means anything.
//! 4. [`Source`]'s three arms decide one dialog flag and the `WARN` records
//!    (spec §13).
//! 5. User writes with the Run and System also needs elevation — the one `&&`
//!    ADR-0002 calls a trap when it is wrong.
//! 6. The Read-only reason survives into the UI, which names it (spec §10.1
//!    item 7, §12).
//! 7. A Scope whose startup read *fails* becomes an empty **non-writable**
//!    Session, because nothing may be written over a value that was never read.
//!
//! **The seam is the OS call, not the crate boundary** (ADR-0010). [`decide`]
//! takes the located directory, the elevation answer, the system language and
//! the two [`ScopeKey`]s — the facts a test cannot make fail — and performs
//! everything downstream of them, which is the same shape [`datadir::decide`]
//! and [`crate::locale::from_langid`] already have. A test then aims the whole
//! sequence at a temporary directory, a temporary registry key and both
//! elevation answers, without needing a privilege or a real machine.
//!
//! **What stays in `main` is assembly, not decisions.** `main` is the
//! composition root: it calls `current_exe()` and `locate`, installs the
//! Catalogue, wraps the Sessions, opens the window and returns the exit code.
//! Moving that here would relocate code without making anything testable, since
//! none of it decides anything.
//!
//! **The records are returned rather than written**, as the Apply Run's are
//! (ADR-0008) — so what a startup logs is assertable without a filesystem.
//! Deciding *whether there is a log at all* stays here, because that is rule
//! one.

use std::cell::RefCell;
use std::path::{Path, PathBuf};

use pathmaster_core::language::{self, Language, SystemLanguage};
use pathmaster_core::logfmt::Record;
use pathmaster_core::session::{Scope, ScopeValue, Session};
use pathmaster_core::settings::SettingsFile;

use crate::datadir::{self, DataDirState, ReadOnlyReason};
use crate::logwriter::Logger;
use crate::panic_hook;
use crate::registry::{RawValue, ScopeKey};
use crate::settings::{self, Source};

/// The facts of this Run an Apply needs, decided once at startup and held by
/// the window for as long as the Run lasts (ADR-0008, ADR-0010).
///
/// It is `Run` because that is what `CONTEXT.md` calls the thing it describes —
/// and it is why the Apply sequence's own per-pass type is [`ApplyRun`], not
/// `Run`: the window holds both, and one of them wearing the other's name is
/// the collision ADR-0010 exists to prevent.
///
/// They travel as one struct rather than as three more parameters on
/// `build_main_window`, because tickets 15 and 17 add two more. The **backup
/// budget is deliberately not among them**: `maxBackups` changes while the
/// application is running, so it is not a property of the Run at all — the
/// window holds the current [`SettingsFile`] and each Apply Run reads the
/// budget from it (ADR-0010).
///
/// [`ApplyRun`]: crate::apply::ApplyRun
pub struct Run {
    /// The Run's log. Behind a `RefCell` because writing a record needs `&mut`
    /// and every caller reaches this through an `Rc<App>` — the same interior
    /// mutability the Sessions have, and with the same rule: no borrow is held
    /// across a call that can run someone else's code.
    logger: RefCell<Logger>,
    /// `DataDirState::dir()` — the located directory whatever this Run may do
    /// with it. Never the path only obtainable by matching `Writable`: startup
    /// predicts, Apply verifies (ADR-0002).
    data_dir: Option<PathBuf>,
    /// Where the log file is, for the one record that cannot ride an outcome:
    /// the broadcast's `WARN`, appended by a thread that may still be blocked
    /// when the Apply that started it has long since returned (spec §4).
    log_path: Option<PathBuf>,
}

impl Run {
    fn new(logger: Logger, data_dir: Option<PathBuf>) -> Run {
        Run {
            log_path: logger.path().map(Path::to_path_buf),
            logger: RefCell::new(logger),
            data_dir,
        }
    }

    /// Writes one record. Nothing here can fail outward — a Run without a log
    /// simply drops it (spec §14).
    pub fn log(&self, record: &Record) {
        self.logger.borrow_mut().log(record);
    }

    pub fn data_dir(&self) -> Option<&Path> {
        self.data_dir.as_deref()
    }

    pub fn log_path(&self) -> Option<&Path> {
        self.log_path.as_deref()
    }
}

/// One Scope as startup decided it: the Session to edit, and the value the
/// registry held when it was read.
///
/// The second is the comparison subject external-change detection needs, and it
/// cannot live in the Session: `RawValue` is a `pathmaster-platform` type and
/// `pathmaster-core` may not reach it (ADR-0008). Both existing readers of a
/// `ScopeKey::read` used to decode it and drop it, which is why the primitive
/// spec §4 fixes had nothing to compare against.
///
/// The Session arrives unwrapped. Sharing it through an `Rc<RefCell<…>>` is the
/// window's business and the window's lifetime, and this crate has no reason to
/// know either.
pub struct LoadedScope {
    pub session: Session,
    pub last_read: RawValue,
}

/// Everything [`decide`] settled, as `main` destructures it.
///
/// One struct rather than a tuple because seven positions are six chances to
/// swap two of them, and two of these are `bool`s.
pub struct Startup {
    /// The facts the window holds for as long as the Run lasts.
    pub run: Run,
    /// What this startup earned, in order — **returned rather than written**
    /// (ADR-0008), so it is assertable without a filesystem. A Run without a
    /// log still earns them; writing them is `main`'s line, and dropping them
    /// is the [`Logger`]'s decision, not this module's.
    pub records: Vec<Record>,
    /// The Interface Language, decided once per Run and never again (spec §11).
    pub language: Language,
    /// The reason this Run cannot write its Data Directory, for the UI that
    /// names it (spec §10.1 item 7, §12) — `None` in Writable Data.
    pub readonly: Option<ReadOnlyReason>,
    /// The settings this Run uses. The window holds them and replaces them when
    /// the Settings dialog does, which is what keeps `maxBackups` out of
    /// [`Run`] (ADR-0010).
    pub settings: SettingsFile,
    /// Whether the user is owed the one startup dialog `settings.json` can cost
    /// them: the file existed and could not be read (spec §13).
    pub settings_unreadable: bool,
    pub user: LoadedScope,
    pub system: LoadedScope,
}

/// Decides everything this Run is, from the facts only the running process can
/// answer.
///
/// Startup order (spec §11): Data Directory → settings → translations → UI.
/// What happens here is that order up to the UI — establish, read the settings,
/// resolve the language, decide writability, load the Sessions — and the seven
/// rules this module's own documentation lists are the joins between them.
///
/// `located` is what [`datadir::locate`] answered for this executable, which is
/// the one call a test cannot make fail; every parameter after it is the same
/// kind of thing. Read-only Data, an unelevated process and an unreadable Scope
/// are all reachable from here without a privilege.
pub fn decide(
    located: Option<PathBuf>,
    elevated: bool,
    system_language: SystemLanguage,
    user_key: &ScopeKey,
    system_key: &ScopeKey,
) -> Startup {
    let data = datadir::decide(located);

    // Rule one. The log lives in the Data Directory, so Read-only Data is a Run
    // without a log — and an unopenable log stays a Run without a log, never
    // Read-only Data (spec §14).
    let logger = match &data {
        DataDirState::Writable(dir) => Logger::open(dir),
        DataDirState::ReadOnly(_) => Logger::disabled(),
    };
    // Rule two. A Run with no log has nowhere for a panic line either.
    if let Some(log_path) = logger.path() {
        panic_hook::install(log_path.to_path_buf());
    }

    // Settings are read in both data modes; only the set-aside of an unreadable
    // file is withheld in Read-only Data.
    let loaded = settings::read(&data);
    // The Interface Language comes from the stored choice, which is a choice and
    // not its outcome (spec §11).
    let language = language::resolve(loaded.file.language(), system_language);

    // Rule three. The startup line comes first even though the settings lines
    // describe something that happened before it: it names the build, and until
    // it does, nothing under it means anything. Read in order, the pair explains
    // itself — `language: en` above, and below it the reason it is not what the
    // file asked for. The version is `env!` here rather than a parameter because
    // the workspace pins one version for all three crates: this is the binary's.
    let mut records = vec![Record::startup(
        env!("CARGO_PKG_VERSION"),
        elevated,
        data.log_state(),
        language.code(),
    )];
    // Rule four.
    let settings_unreadable = match &loaded.source {
        // An absent file is a first run, not a failure: no dialog, no log line.
        Source::Absent => false,
        // A bad field is noise, and the log is its only witness (spec §13).
        Source::Read(rejected) => {
            records.extend(rejected.iter().map(|rejection| rejection.record()));
            false
        }
        // An unreadable file is "your edit did not take" — the user is owed a
        // dialog, and a developer reading the log is owed the file's whereabouts.
        Source::Unreadable { set_aside } => {
            records.push(Record::settings_unreadable(*set_aside));
            true
        }
    };

    // Rule five. The two Editing Sessions, one per Scope (spec §5), loaded from
    // raw registry reads through the adapter. Writability is decided from the
    // Run's own facts: User writes with the Run, System also needs elevation,
    // and Read-only Data closes both (spec §3, §9).
    let data_writable = data.is_writable();
    let user = load_session(Scope::User, user_key, data_writable, &mut records);
    let system = load_session(
        Scope::System,
        system_key,
        data_writable && elevated,
        &mut records,
    );

    // Rule six. Announcement 7 and StatusBar field 0 name the reason.
    let readonly = match &data {
        DataDirState::Writable(_) => None,
        DataDirState::ReadOnly(reason) => Some(reason.clone()),
    };

    Startup {
        run: Run::new(logger, data.dir().map(Path::to_path_buf)),
        records,
        language,
        readonly,
        settings: loaded.file,
        settings_unreadable,
        user,
        system,
    }
}

/// Rule seven. One Scope's startup read, decoded into its Session with the
/// Baseline set — Absent decodes to zero Entries (spec §5). The raw value is
/// kept beside it: external-change detection compares `(vtype, bytes)`, and
/// decoding stops at the first NUL, so a decoded copy would miss a real change
/// (spec §4).
///
/// The spec never names a failed startup read, so it takes the degraded road:
/// an empty, *non-writable* Session — nothing may be written over a value that
/// was never read — with the log line as the developer's only witness. Its
/// last-read value is `Absent`, which nothing ever compares: a non-writable
/// Session has no Apply to reach it.
fn load_session(
    scope: Scope,
    key: &ScopeKey,
    writable: bool,
    records: &mut Vec<Record>,
) -> LoadedScope {
    let (value, raw, writable) = match key.read() {
        Ok(raw) => (raw.decode(), raw, writable),
        Err(err) => {
            records.push(Record::scope_read_failed(scope, err.log_cause()));
            (ScopeValue::Absent, RawValue::Absent, false)
        }
    };
    LoadedScope {
        session: Session::new(scope, value, writable),
        last_read: raw,
    }
}
