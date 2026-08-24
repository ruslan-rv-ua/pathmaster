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
mod pump;
mod ui;
mod version;

use std::cell::RefCell;
use std::rc::Rc;

use pathmaster_core::session::Session;
use pathmaster_platform::datadir;
use pathmaster_platform::elevation;
use pathmaster_platform::locale;
use pathmaster_platform::registry::{RawValue, ScopeKey};
use pathmaster_platform::startup::{self, Decisions, LoadedScope};

/// One Scope as the window holds it: the Session, shared, and the value the
/// registry held when startup read it.
///
/// The sharing is all this adds to [`LoadedScope`]. It belongs here because it
/// is the window that needs it — every command reaches its Session from a
/// closure holding an `Rc<App>` — and `pathmaster-platform` has no reason to
/// know that.
pub struct SharedScope {
    pub session: Rc<RefCell<Session>>,
    pub last_read: RawValue,
}

impl From<LoadedScope> for SharedScope {
    /// Wrapping one Session for the window, which is the whole of what
    /// "assembly" means here: `Rc` because every command holds the window,
    /// `RefCell` because editing needs `&mut` — under the standing rule that no
    /// borrow is held across a call that can run someone else's code.
    fn from(loaded: LoadedScope) -> SharedScope {
        SharedScope {
            session: Rc::new(RefCell::new(loaded.session)),
            last_read: loaded.last_read,
        }
    }
}

fn main() -> std::process::ExitCode {
    // Everything this Run is, decided in one place (ADR-0010). What cannot be
    // decided there is asked here, at the edge, because these are the calls no
    // test can make fail: where this executable is, whether its process is
    // elevated, and what language Windows shows its own interface in.
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
        std::env::current_exe()
            .ok()
            .as_deref()
            .and_then(datadir::locate),
        elevation::is_elevated(),
        locale::system_language(),
        &ScopeKey::user(),
        &ScopeKey::system(),
    );
    // Decided there, written here — the shape an Apply Run's records already
    // have (ADR-0008). A Run without a log drops them.
    records.iter().for_each(|record| run.log(record));

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
            // The one argument an elevation relaunch carries: the tab the
            // user left (spec §9, ticket 12 D5). Read at the edge like every
            // other fact only the running process can answer. Lossily,
            // because `std::env::args` panics on non-Unicode arguments — a
            // launcher's garbage must read as a plain launch, not a crash.
            elevation::StartTab::from_args(
                std::env::args_os()
                    .skip(1)
                    .map(|arg| arg.to_string_lossy().into_owned()),
            ),
        );
        if settings_unreadable {
            ui::show_settings_unreadable(&frame);
        }
    }) {
        Ok(()) => std::process::ExitCode::SUCCESS,
        Err(_) => std::process::ExitCode::FAILURE,
    }
}
