//! PathMaster — portable Windows PATH editor, built for an NVDA user first.
//!
//! The GUI shell is covered by the Release Checklist, never by automated tests
//! (spec §18, ADR-0007). Accessibility rides the free native comctl32 path:
//! zero `set_accessibility_*` calls anywhere (ADR-0003), and nothing sets a colour.

#![windows_subsystem = "windows"]

mod announce;
mod catalog;
mod pump;
mod ui;

use std::cell::RefCell;
use std::path::{Path, PathBuf};
use std::rc::Rc;

use pathmaster_core::language;
use pathmaster_core::logfmt::Record;
use pathmaster_core::session::{Scope, ScopeValue, Session};
use pathmaster_platform::datadir::{self, DataDirState};
use pathmaster_platform::elevation;
use pathmaster_platform::locale;
use pathmaster_platform::logwriter::Logger;
use pathmaster_platform::panic_hook;
use pathmaster_platform::registry::{RawValue, ScopeKey};
use pathmaster_platform::settings::{self, Source};

/// The facts of this Run an Apply needs, decided once at startup and held by
/// the window for as long as the Run lasts (ADR-0008).
///
/// They travel as one struct rather than as three more parameters on
/// `build_main_window`, because tickets 15 and 17 add two more. The **backup
/// budget is deliberately not among them**: `maxBackups` changes while the
/// application is running, so it is not a property of the Run at all — the
/// window holds the current `SettingsFile` and each Apply Run reads the budget
/// from it (ADR-0010).
pub struct RunFacts {
    /// The run's log. Behind a `RefCell` because writing a record needs `&mut`
    /// and every caller reaches this through an `Rc<App>` — the same interior
    /// mutability the Sessions have, and with the same rule: no borrow is held
    /// across a call that can run someone else's code.
    logger: RefCell<Logger>,
    /// `DataDirState::dir()` — the located directory whatever this run may do
    /// with it. Never the path only obtainable by matching `Writable`: startup
    /// predicts, Apply verifies (ADR-0002).
    data_dir: Option<PathBuf>,
    /// Where the log file is, for the one record that cannot ride an outcome:
    /// the broadcast's `WARN`, appended by a thread that may still be blocked
    /// when the Apply that started it has long since returned (spec §4).
    log_path: Option<PathBuf>,
}

impl RunFacts {
    fn new(logger: Logger, data_dir: Option<PathBuf>) -> RunFacts {
        RunFacts {
            log_path: logger.path().map(Path::to_path_buf),
            logger: RefCell::new(logger),
            data_dir,
        }
    }

    /// Writes one record. Nothing here can fail outward — a run without a log
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

/// One Scope as startup hands it to the window: the Session to edit, and the
/// value the registry held when it was read.
///
/// The second is the comparison subject external-change detection needs, and
/// it cannot live in the Session: `RawValue` is a `pathmaster-platform` type
/// and `pathmaster-core` may not reach it (ADR-0008). Both existing readers of
/// a `ScopeKey::read` used to decode it and drop it, which is why the primitive
/// spec §4 fixes had nothing to compare against.
pub struct LoadedScope {
    pub session: Rc<RefCell<Session>>,
    pub last_read: RawValue,
}

fn main() -> std::process::ExitCode {
    // The startup facts, each decided exactly once per run (spec §3, §9):
    // the Data Directory state and elevation are properties of the run.
    let data = datadir::startup();
    let elevated = elevation::is_elevated();

    // The log lives in the Data Directory, so Read-only Data is a run without
    // a log — and an unopenable log stays a run without a log, never
    // Read-only Data (spec §14).
    let mut logger = match &data {
        DataDirState::Writable(dir) => Logger::open(dir),
        DataDirState::ReadOnly(_) => Logger::disabled(),
    };
    if let Some(log_path) = logger.path() {
        panic_hook::install(log_path.to_path_buf());
    }

    // Startup order (spec §11): Data Directory → settings → translations → UI.
    // Settings are read in both data modes; only the set-aside of an unreadable
    // file is withheld in Read-only Data.
    let loaded = settings::read(&data);
    // Interface Language, decided once per run and never again (spec §11) —
    // from the stored choice, which is a choice and not its outcome.
    let language = language::resolve(loaded.file.language(), locale::system_language());

    // The startup line comes first even though the settings lines describe
    // something that happened before it: it names the build, and until it does,
    // nothing under it means anything. Read in order, the pair explains itself
    // — `language: en` above, and below it the reason it is not what the file
    // asked for.
    logger.log(&Record::startup(
        env!("CARGO_PKG_VERSION"),
        elevated,
        data.log_state(),
        language.code(),
    ));
    let settings_unreadable = match &loaded.source {
        // An absent file is a first run, not a failure: no dialog, no log line.
        Source::Absent => false,
        // A bad field is noise, and the log is its only witness (spec §13).
        Source::Read(rejected) => {
            rejected.iter().for_each(|r| logger.log(&r.record()));
            false
        }
        // An unreadable file is "your edit did not take" — the user is owed a
        // dialog, and a developer reading the log is owed the file's whereabouts.
        Source::Unreadable { set_aside } => {
            logger.log(&Record::settings_unreadable(*set_aside));
            true
        }
    };

    // The two Editing Sessions, one per Scope (spec §5), loaded from raw
    // registry reads through the adapter. Writability is decided here, from
    // the run's startup facts: User writes with the run, System also needs
    // elevation, and Read-only Data closes both (spec §3, §9).
    let data_writable = data.is_writable();
    let user_scope = load_session(Scope::User, &ScopeKey::user(), data_writable, &mut logger);
    let system_scope = load_session(
        Scope::System,
        &ScopeKey::system(),
        data_writable && elevated,
        &mut logger,
    );
    // The reason survives into the UI: Announcement 7 and StatusBar field 0
    // name it (spec §10.1 item 7, §12).
    let readonly = match &data {
        DataDirState::Writable(_) => None,
        DataDirState::ReadOnly(reason) => Some(reason.clone()),
    };
    let facts = RunFacts::new(logger, data.dir().map(Path::to_path_buf));

    // No console to print to (windows subsystem) — a failed toolkit init can
    // only surface as a nonzero exit code (and the panic line, if it panics).
    match wxdragon::main(move |_| {
        catalog::install(language);
        let frame = ui::build_main_window(user_scope, system_scope, readonly, facts, loaded.file);
        if settings_unreadable {
            ui::show_settings_unreadable(&frame);
        }
    }) {
        Ok(()) => std::process::ExitCode::SUCCESS,
        Err(_) => std::process::ExitCode::FAILURE,
    }
}

/// One Scope's startup read, decoded into its Session with the Baseline set —
/// Absent decodes to zero Entries (spec §5). The raw value is kept beside it:
/// external-change detection compares `(vtype, bytes)`, and decoding stops at
/// the first NUL, so a decoded copy would miss a real change (spec §4).
///
/// The spec never names a failed startup read, so it takes the degraded road:
/// an empty, *non-writable* Session — nothing may be written over a value that
/// was never read — with the log line as the developer's only witness. Its
/// last-read value is `Absent`, which nothing ever compares: a non-writable
/// Session has no Apply to reach it.
fn load_session(scope: Scope, key: &ScopeKey, writable: bool, logger: &mut Logger) -> LoadedScope {
    let (value, raw, writable) = match key.read() {
        Ok(raw) => (raw.decode(), raw, writable),
        Err(err) => {
            logger.log(&Record::scope_read_failed(scope, err.log_cause()));
            (ScopeValue::Absent, RawValue::Absent, false)
        }
    };
    LoadedScope {
        session: Rc::new(RefCell::new(Session::new(scope, value, writable))),
        last_read: raw,
    }
}
