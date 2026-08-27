//! PathMaster — portable Windows PATH editor, built for an NVDA user first.
//!
//! The GUI shell is covered by the Release Checklist, never by automated tests
//! (spec §18, ADR-0007). Accessibility rides the free native comctl32 path:
//! zero `set_accessibility_*` calls anywhere (ADR-0003), and nothing sets a colour.
//!
//! `main` is the composition root and **only** that: what a Run is — where its
//! data lives and whether it can be written, whether it is elevated, what
//! language it speaks, whether it has a log, which Scopes it may write — is
//! decided by [`startup::decide`], and what is left here is assembly (ADR-0010).

#![windows_subsystem = "windows"]

mod announce;
mod catalog;
mod clipboard;
mod pump;
mod scoped;
mod ui;

use std::rc::Rc;

use pathmaster_core::language::{self, LanguageChoice, SystemLanguage};
use pathmaster_core::logfmt::Record;
use pathmaster_core::session::Session;
use pathmaster_platform::args::Arguments;
use pathmaster_platform::datadir::{self, Location};
use pathmaster_platform::elevation;
use pathmaster_platform::locale;
use pathmaster_platform::registry::{RawValue, ScopeKey};
use pathmaster_platform::startup::{self, Decisions, LoadedScope};

use crate::scoped::Scoped;

/// One Scope as the window holds it: the Session, shared, and the value the
/// registry held when startup read it.
///
/// The sharing is all this adds to [`LoadedScope`]. It belongs here because it
/// is the window that needs it — every command reaches its Session from a
/// closure holding an `Rc<App>` — and `pathmaster-platform` has no reason to
/// know that.
pub struct SharedScope {
    pub session: Rc<Scoped<Session>>,
    pub last_read: RawValue,
}

impl From<LoadedScope> for SharedScope {
    /// Wrapping one Session for the window, which is the whole of what
    /// "assembly" means here: `Rc` because every command holds the window,
    /// [`Scoped`] because editing needs `&mut` and a Session is state more
    /// than one kind of call reaches (ADR-0011).
    fn from(loaded: LoadedScope) -> SharedScope {
        SharedScope {
            session: Rc::new(Scoped::new(loaded.session)),
            last_read: loaded.last_read,
        }
    }
}

fn main() -> std::process::ExitCode {
    // The command line, read once and in full before anything else happens —
    // it decides *whether* there is a Run at all, and where one would write
    // (v0.2.0 §10). Read as `OsString`, because one of its arguments is a
    // filesystem path and a lossy reading of that is a path to somewhere else.
    let arguments = Arguments::parse(std::env::args_os().skip(1));

    // A query, not a launch: answered and then over, with no Data Directory
    // located, created or read.
    if arguments.help {
        return answer_the_query(locale::system_language());
    }

    // Everything this Run is, decided in one place (ADR-0010). What cannot be
    // decided there is asked here, at the edge, because these are the calls no
    // test can make fail: where this executable is, what the current directory
    // is, whether this process is elevated, and what language Windows shows its
    // own interface in.
    let Decisions {
        run,
        records,
        language,
        readonly,
        settings,
        settings_unreadable,
        user,
        system,
    } = startup::decide(
        locate_this_run(&arguments),
        elevation::is_elevated(),
        locale::system_language(),
        &ScopeKey::user(),
        &ScopeKey::system(),
    );
    // Decided there, written here — the shape an Apply Run's records already
    // have (ADR-0008). A Run without a log drops them.
    records.iter().for_each(|record| run.log(record));
    // One `WARN` per unknown argument, under the startup line that says which
    // build ignored them (v0.2.0 §10). The dialog below names only the first;
    // this is the inventory.
    for argument in &arguments.unknown {
        run.log(&Record::unknown_argument(&argument.to_string_lossy()));
    }
    let unknown = arguments
        .unknown
        .first()
        .map(|argument| argument.to_string_lossy().into_owned());

    // No console to print to (windows subsystem) — a failed toolkit init can
    // only surface as a nonzero exit code (and the panic line, if it panics).
    match wxdragon::main(move |_| {
        catalog::install(language);
        let frame = ui::build_main_window(
            SharedScope::from(user),
            SharedScope::from(system),
            readonly,
            run,
            settings,
            // The tab an elevation relaunch carries (spec §9, ticket 12 D5).
            arguments.tab,
        );
        // The command line's dialog before the Run's own: this one is about
        // how the application was started, and that is the earlier fact.
        if let Some(argument) = &unknown {
            ui::show_unknown_argument(&frame, argument);
        }
        if settings_unreadable {
            ui::show_settings_unreadable(&frame);
        }
    }) {
        Ok(()) => std::process::ExitCode::SUCCESS,
        Err(_) => std::process::ExitCode::FAILURE,
    }
}

/// The locate step, and the one thing `--data-dir` substitutes (v0.2.0 §10).
///
/// Assembly, not a decision: the two OS calls are here at the edge because no
/// test can make them fail, and what each answer *means* is decided by
/// [`datadir::locate`] and [`datadir::locate_override`], which are pure.
///
/// A current directory this process cannot name leaves an empty one, which is
/// exactly right — an absolute `--data-dir` needs none, and a relative one
/// cannot be resolved without it and is a broken override, which is what
/// joining onto nothing produces.
fn locate_this_run(arguments: &Arguments) -> Location {
    if let Some(value) = &arguments.data_dir {
        let cwd = std::env::current_dir().unwrap_or_default();
        return datadir::locate_override(value, &cwd);
    }
    std::env::current_exe()
        .ok()
        .as_deref()
        .and_then(datadir::locate)
        .map_or(Location::OwnLocationUnknown, Location::BesideExe)
}

/// `--help` / `-?`: the usage dialog, and no Run at all (v0.2.0 §10).
///
/// wx is started because the Catalogue lives inside it and this answer is
/// Catalogue text; nothing else about a Run is. The language is the system's,
/// for the reason [`ui::show_usage`] gives.
fn answer_the_query(system_language: SystemLanguage) -> std::process::ExitCode {
    match wxdragon::main(move |_| {
        catalog::install(language::resolve(LanguageChoice::Auto, system_language));
        ui::show_usage();
    }) {
        Ok(()) => std::process::ExitCode::SUCCESS,
        Err(_) => std::process::ExitCode::FAILURE,
    }
}
