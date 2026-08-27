//! What the command line says, and how this application writes one
//! (v0.2.0 §10).
//!
//! Three switches and one posture. `--tab` is v0.1.0's, carried across the
//! elevation boundary; `--data-dir` points one Run's Data Directory somewhere
//! else; `--help` (and `-?`) is a query rather than a launch. Anything else is
//! an **unknown argument**: named in a dialog, logged, and then ignored — never
//! a refusal to start, and never a silent ignoring, because a typo'd
//! `--datadir` that quietly landed data in the default directory is the exact
//! hazard `--data-dir` exists to prevent.
//!
//! **Arguments are `OsString` all the way through.** `to_string_lossy` would
//! replace an unpaired surrogate with U+FFFD, and for the one argument that is
//! a filesystem path that means pointing the application *near* where the user
//! pointed it. `--tab`'s v0.1.0 reader could afford lossy — a mangled value is
//! not one of three known words, so it reads as a plain launch — and this one
//! cannot.
//!
//! **Writer and reader are one type.** [`StartTab::argument`] is the v0.1.0
//! trick: the spawner and the reader share one function, so the two instances
//! cannot drift about a spelling. [`CommandLine`] extends it to the whole line
//! — an elevated relaunch is built from *parsed state*, never from the verbatim
//! command-line tail, and read back by the same rules it was written by.

use std::ffi::{OsStr, OsString};
use std::os::windows::ffi::{OsStrExt, OsStringExt};

use crate::datadir::Location;

/// The switch that carries the tab across the elevation boundary (spec §9,
/// ADR-0005).
pub const SWITCH_TAB: &str = "--tab";

/// The switch that points one Run's Data Directory elsewhere (v0.2.0 §10).
pub const SWITCH_DATA_DIR: &str = "--data-dir";

/// The two spellings of the query. `-?` is the cmd.exe convention and `--help`
/// the one everything else answers to; both do the same thing.
pub const SWITCH_HELP: &str = "--help";
pub const SWITCH_HELP_SHORT: &str = "-?";

/// The tab the user left, named for the relaunch's one v0.1.0 argument (ticket
/// 12 D5). It is a *tab* and not a Scope because the Backups tab is one of the
/// places the user can be — `CONTEXT.md` keeps "Tab" off **Scope** for exactly
/// this reason.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StartTab {
    User,
    System,
    Backups,
}

impl StartTab {
    /// Every tab, for the callers that search rather than match — the parser
    /// below, and the window's inverse lookup.
    pub const ALL: [StartTab; 3] = [StartTab::User, StartTab::System, StartTab::Backups];

    /// The value the spawner writes after `--tab`. [`Arguments::parse`] reads
    /// by searching this same function over [`ALL`](Self::ALL), so the pair
    /// round-trips by construction — there is no second spelling to drift.
    pub fn argument(self) -> &'static str {
        match self {
            StartTab::User => "user",
            StartTab::System => "system",
            StartTab::Backups => "backups",
        }
    }
}

/// Everything this Run's command line asked for.
///
/// Four fields and no errors: no argument this application can be given is
/// fatal, so the parse always answers, and what it answers with is what each
/// surface then does about it. A malformed `--data-dir` is the sharpest case —
/// it is *not* an unknown argument but a broken override, which the locate step
/// answers with Read-only Data rather than the parser answering with a dialog.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Arguments {
    /// The tab to open on, when the line named one this application's own
    /// spawner writes. `None` is a plain launch — including for a `--tab` whose
    /// value is a word nothing writes, which is v0.1.0's leniency and stays:
    /// the caller's default is a better answer than a guess.
    pub tab: Option<StartTab>,
    /// The value `--data-dir` carried, exactly as the shell delivered it.
    /// `None` when the switch is absent, and **empty** when it was given with
    /// nothing after it. A valueless switch and an empty value are one state
    /// here because §10 gives them one outcome: a broken override.
    pub data_dir: Option<OsString>,
    /// Whether the line asked to be told about itself rather than to launch
    /// anything.
    pub help: bool,
    /// Every argument this application does not recognise, in the order given.
    /// The first earns the dialog; every one of them earns a `WARN` line.
    pub unknown: Vec<OsString>,
}

impl Arguments {
    /// Reads a command line's arguments — `argv[1..]`, never the program name.
    ///
    /// A switch's *last* occurrence is the one that counts. Repetition is
    /// pathological (this application's own spawner writes each switch once),
    /// and one rule for both switches beats two rules to remember.
    ///
    /// `--data-dir` takes the next argument whatever it looks like, the way
    /// every value-taking option does: a value that begins with `-` is a
    /// directory with an odd name, not evidence that the user meant something
    /// else, and inventing a rule to tell those apart would guess at exactly
    /// the thing this switch may not guess at.
    pub fn parse(args: impl IntoIterator<Item = OsString>) -> Arguments {
        let mut parsed = Arguments::default();
        let mut args = args.into_iter();
        while let Some(arg) = args.next() {
            if is_switch(&arg, SWITCH_HELP) || is_switch(&arg, SWITCH_HELP_SHORT) {
                parsed.help = true;
            } else if is_switch(&arg, SWITCH_TAB) {
                parsed.tab = read_tab(&args.next().unwrap_or_default());
            } else if is_switch(&arg, SWITCH_DATA_DIR) {
                parsed.data_dir = Some(args.next().unwrap_or_default());
            } else if let Some(value) = value_after(&arg, SWITCH_DATA_DIR) {
                // The `=` spelling, for `--data-dir` alone: §10 gives it both,
                // and gives `--tab` only the space form its spawner writes.
                parsed.data_dir = Some(value);
            } else {
                parsed.unknown.push(arg);
            }
        }
        parsed
    }
}

/// Whether this argument is that switch, spelled exactly.
fn is_switch(arg: &OsStr, switch: &str) -> bool {
    arg == OsStr::new(switch)
}

/// One tab value, read by the same function that writes it — v0.1.0's leniency
/// intact: a value nothing writes is no request at all, and it is still
/// `--tab`'s value, so it is not an unknown argument either.
fn read_tab(value: &OsStr) -> Option<StartTab> {
    StartTab::ALL
        .into_iter()
        .find(|tab| value == OsStr::new(tab.argument()))
}

/// The value of a `--switch=value` spelling, exactly as the shell delivered it,
/// or `None` when `arg` is not that switch.
///
/// Compared as UTF-16 units rather than through `to_string_lossy`, for the
/// reason the module doc gives: this is how a Data Directory path arrives.
fn value_after(arg: &OsStr, switch: &str) -> Option<OsString> {
    const EQUALS: u16 = b'=' as u16;

    let prefix: Vec<u16> = switch.encode_utf16().chain([EQUALS]).collect();
    let units: Vec<u16> = arg.encode_wide().collect();
    units
        .strip_prefix(prefix.as_slice())
        .map(OsString::from_wide)
}

/// One command line, in the two directions a self-relaunch needs it
/// (v0.2.0 §10).
///
/// `ShellExecuteExW` takes a hand-built `lpParameters` string, which
/// `std::process::Command`'s quoting never reaches — so the quoting is ours,
/// and it is the documented Windows one (Colascione's rules): a run of *n*
/// backslashes doubles to *2n* before a quote this writer adds, and to *2n+1*
/// before a quote the value itself carries. Anywhere else they are literal,
/// which is why `C:\Program Files\` needs the doubling only at its very end.
///
/// [`split`](Self::split) is the same rules read backwards. It exists so the
/// pair can be round-tripped in a unit test rather than by spawning a process:
/// what actually parses the elevated instance's line is Windows itself, and a
/// writer with no reader beside it is a writer nothing measures.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct CommandLine {
    units: Vec<u16>,
}

impl CommandLine {
    const QUOTE: u16 = b'"' as u16;
    const BACKSLASH: u16 = b'\\' as u16;
    const SPACE: u16 = b' ' as u16;
    const TAB: u16 = b'\t' as u16;
    const NEWLINE: u16 = b'\n' as u16;
    const VERTICAL_TAB: u16 = 0x0b;

    /// The line a self-relaunch carries: **re-serialized from parsed state**,
    /// never the verbatim tail of this process's own command line
    /// (v0.2.0 §10).
    ///
    /// The tail would carry the user's original spelling of a relative path,
    /// and the elevated process's current directory is not guaranteed to be
    /// this one's — so what crosses is the *resolved absolute* path this Run
    /// settled on at startup. It would also carry any unknown argument along
    /// with it, and those die at this boundary: the elevated instance is not
    /// owed a second dialog about a typo the user has already dismissed.
    ///
    /// A broken override crosses as the bare switch it was. That looks odd
    /// written down and is the only honest answer: dropping it would land the
    /// elevated instance in the default `data\` — writing where it was not
    /// pointed, which is the one thing §10 forbids outright.
    pub fn relaunch(tab: StartTab, location: &Location) -> CommandLine {
        let mut line = CommandLine::default();
        line.push(OsStr::new(SWITCH_TAB));
        line.push(OsStr::new(tab.argument()));
        match location {
            Location::Override(dir) => {
                line.push(OsStr::new(SWITCH_DATA_DIR));
                line.push(dir.as_os_str());
            }
            Location::BrokenOverride => line.push(OsStr::new(SWITCH_DATA_DIR)),
            Location::BesideExe(_) | Location::OwnLocationUnknown => {}
        }
        line
    }

    /// Appends one argument, quoted only where quoting changes what it means.
    pub fn push(&mut self, argument: &OsStr) {
        if !self.units.is_empty() {
            self.units.push(Self::SPACE);
        }
        let value: Vec<u16> = argument.encode_wide().collect();
        if !value.is_empty() && !value.iter().any(|unit| Self::needs_quoting(*unit)) {
            self.units.extend_from_slice(&value);
            return;
        }
        self.units.push(Self::QUOTE);
        let mut rest = value.as_slice();
        while let Some((unit, tail)) = rest.split_first() {
            match *unit {
                Self::BACKSLASH => {
                    let run = 1 + tail.iter().take_while(|u| **u == Self::BACKSLASH).count();
                    rest = &rest[run..];
                    // Doubled only where a quote follows — the one the value
                    // carries, or the one this writer is about to close with.
                    let doubled = rest.first().is_none_or(|next| *next == Self::QUOTE);
                    let emit = if doubled { run * 2 } else { run };
                    self.units
                        .extend(std::iter::repeat_n(Self::BACKSLASH, emit));
                }
                Self::QUOTE => {
                    self.units.push(Self::BACKSLASH);
                    self.units.push(Self::QUOTE);
                    rest = tail;
                }
                other => {
                    self.units.push(other);
                    rest = tail;
                }
            }
        }
        self.units.push(Self::QUOTE);
    }

    /// The line as `ShellExecuteExW` and the log both want to see it.
    pub fn into_os_string(self) -> OsString {
        OsString::from_wide(&self.units)
    }

    /// Reads a command line back into its arguments, by the rules
    /// [`push`](Self::push) writes them under — Windows' own, minus the special
    /// treatment `argv[0]` gets, which `lpParameters` never carries.
    ///
    /// Deliberately a *model* of `CommandLineToArgvW` rather than a call to it:
    /// the point is to measure this type's own two halves against each other,
    /// and a round trip through the very function the writer is aimed at is the
    /// assertion, not the implementation.
    pub fn split(line: &OsStr) -> Vec<OsString> {
        let mut arguments = Vec::new();
        let mut current: Vec<u16> = Vec::new();
        let mut started = false;
        let mut quoted = false;
        let mut backslashes = 0usize;

        for unit in line.encode_wide() {
            match unit {
                Self::BACKSLASH => {
                    backslashes += 1;
                    continue;
                }
                Self::QUOTE => {
                    current.extend(std::iter::repeat_n(Self::BACKSLASH, backslashes / 2));
                    if backslashes % 2 == 1 {
                        current.push(Self::QUOTE);
                    } else {
                        quoted = !quoted;
                    }
                    backslashes = 0;
                    started = true;
                    continue;
                }
                _ => {}
            }
            current.extend(std::iter::repeat_n(Self::BACKSLASH, backslashes));
            backslashes = 0;
            if !quoted && (unit == Self::SPACE || unit == Self::TAB) {
                if started {
                    arguments.push(OsString::from_wide(&current));
                    current.clear();
                    started = false;
                }
                continue;
            }
            current.push(unit);
            started = true;
        }

        current.extend(std::iter::repeat_n(Self::BACKSLASH, backslashes));
        if started || !current.is_empty() {
            arguments.push(OsString::from_wide(&current));
        }
        arguments
    }

    /// The characters that make an argument one word only because it is quoted.
    /// The empty argument is the other case, and it is handled where the length
    /// is known.
    fn needs_quoting(unit: u16) -> bool {
        matches!(
            unit,
            Self::SPACE | Self::TAB | Self::QUOTE | Self::NEWLINE | Self::VERTICAL_TAB
        )
    }
}
