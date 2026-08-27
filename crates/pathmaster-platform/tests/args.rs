//! The command line, both ways (v0.2.0 §10): what this application reads off
//! one, and what it writes onto one when it relaunches itself.
//!
//! The round trips are the point. `--tab`'s writer and reader have been one
//! function since v0.1.0; `--data-dir` adds a value that can contain spaces and
//! quotes, and an elevated relaunch has to carry it through a hand-built
//! `lpParameters` string. So the assertion that matters is not "the quoting
//! looks right" but "a line written by this type reads back as the arguments it
//! was written from" — for the paths that are awkward on purpose.

#![cfg(windows)]

use std::ffi::{OsStr, OsString};
use std::path::{Path, PathBuf};

use pathmaster_platform::args::{Arguments, CommandLine, StartTab};
use pathmaster_platform::datadir::Location;

/// A command line as a shell would hand it over, argument by argument.
fn parse(args: &[&str]) -> Arguments {
    Arguments::parse(args.iter().map(OsString::from))
}

fn data_dir(parsed: &Arguments) -> Option<&OsStr> {
    parsed.data_dir.as_deref()
}

fn unknown(parsed: &Arguments) -> Vec<String> {
    parsed
        .unknown
        .iter()
        .map(|arg| arg.to_string_lossy().into_owned())
        .collect()
}

// --------------------------------------------------------------------- --tab

/// The writer and the reader of `--tab` are the same function, so what the
/// unelevated instance says is what the elevated one hears — for every tab, the
/// Backups tab included (it is a tab the user can leave, even though it is not
/// a Scope).
#[test]
fn the_tab_argument_round_trips_for_every_tab() {
    for tab in StartTab::ALL {
        let spawned = ["--tab", tab.argument()];
        assert_eq!(parse(&spawned).tab, Some(tab));
    }
}

/// v0.1.0's leniency, intact: anything that is not our own spawn reads as no
/// request at all — a plain launch, a bare `--tab`, a value nothing writes. The
/// degraded answer is `None`, never a guessed tab.
#[test]
fn a_foreign_or_missing_tab_value_reads_as_none() {
    for case in [
        &[][..],
        &["--tab"],
        &["--tab", "banana"],
        &["--tab", ""],
        &["user"],
    ] {
        assert_eq!(parse(case).tab, None, "args: {case:?}");
    }
}

/// A value `--tab` swallowed is that switch's value, however odd it looks — so
/// it is not also an unknown argument. Only a bare `user` is.
#[test]
fn a_tab_value_is_never_also_an_unknown_argument() {
    assert!(unknown(&parse(&["--tab", "banana"])).is_empty());
    assert_eq!(unknown(&parse(&["user"])), ["user"]);
}

// ---------------------------------------------------------------- --data-dir

/// Both spellings, one outcome (§10). The README documents the space form; the
/// `=` form is the one a script is likelier to write.
#[test]
fn both_data_dir_spellings_carry_the_same_value() {
    for case in [
        &["--data-dir", r"D:\PathMaster data"][..],
        &[r"--data-dir=D:\PathMaster data"],
    ] {
        assert_eq!(
            data_dir(&parse(case)),
            Some(OsStr::new(r"D:\PathMaster data")),
            "args: {case:?}"
        );
    }
}

/// A valueless or empty `--data-dir` is a **broken override**, not an unknown
/// argument: the switch was recognised, so the dialog is not the answer — the
/// locate step is, and it answers with Read-only Data.
#[test]
fn a_valueless_data_dir_is_a_broken_override_and_not_an_unknown_argument() {
    for case in [&["--data-dir"][..], &["--data-dir", ""], &["--data-dir="]] {
        let parsed = parse(case);
        assert_eq!(data_dir(&parsed), Some(OsStr::new("")), "args: {case:?}");
        assert!(unknown(&parsed).is_empty(), "args: {case:?}");
    }
}

/// The switch takes the next argument whatever it looks like, the way every
/// value-taking option does. A directory named `--tab` is absurd and legal, and
/// a rule that guessed otherwise would be guessing at exactly the thing this
/// switch may not guess at.
#[test]
fn data_dir_takes_the_next_argument_verbatim() {
    let parsed = parse(&["--data-dir", "--tab", "user"]);
    assert_eq!(data_dir(&parsed), Some(OsStr::new("--tab")));
    assert_eq!(parsed.tab, None);
    assert_eq!(unknown(&parsed), ["user"]);
}

// -------------------------------------------------------------------- --help

#[test]
fn both_help_spellings_are_recognised() {
    for case in [&["--help"][..], &["-?"], &["--tab", "system", "--help"]] {
        assert!(parse(case).help, "args: {case:?}");
    }
    assert!(!parse(&["--tab", "system"]).help);
}

// --------------------------------------------------------- unknown arguments

/// Every unrecognised argument is kept, in the order given: the first earns the
/// dialog and all of them earn a `WARN` line, so the parse may not collapse
/// them.
#[test]
fn unknown_arguments_are_kept_in_order() {
    let parsed = parse(&["--datadir", r"C:\x", "--elevated-write"]);
    assert_eq!(unknown(&parsed), ["--datadir", r"C:\x", "--elevated-write"]);
    assert_eq!(data_dir(&parsed), None);
    assert!(!parsed.help);
}

/// The hazard the whole posture exists for: a typo'd switch must not read as
/// the real one and must not be silently ignored — the run starts, and it says
/// so.
#[test]
fn a_typo_of_the_data_dir_switch_is_unknown_rather_than_obeyed() {
    let parsed = parse(&["--datadir=C:\\x"]);
    assert_eq!(data_dir(&parsed), None);
    assert_eq!(unknown(&parsed), [r"--datadir=C:\x"]);
}

// ------------------------------------------------------- writing a line back

/// The round trip, over the shapes a Data Directory path actually takes on
/// Windows: a space, a trailing backslash (which is what makes quoting hard,
/// since it would escape the closing quote), a quote inside the value, and the
/// empty argument.
#[test]
fn every_awkward_argument_survives_being_written_and_read_back() {
    let cases: &[&[&str]] = &[
        &["--tab", "user"],
        &["--data-dir", r"C:\Program Files\PathMaster"],
        &["--data-dir", r"C:\Program Files\PathMaster\"],
        &["--data-dir", r"C:\ends with two\\"],
        &["--data-dir", r#"C:\a "quoted" name"#],
        &["--data-dir", r#"C:\ends with a quote""#],
        &["--data-dir", ""],
        &["--data-dir", "  "],
        &["--tab", "backups", "--data-dir", r"D:\two words\here\"],
    ];
    for case in cases {
        let mut line = CommandLine::default();
        for argument in *case {
            line.push(OsStr::new(argument));
        }
        let written = line.into_os_string();
        let read_back: Vec<OsString> = CommandLine::split(&written);
        let expected: Vec<OsString> = case.iter().map(OsString::from).collect();
        assert_eq!(read_back, expected, "line: {written:?}");
    }
}

/// And the round trip closes on the parser, not just on the splitter: a
/// spaced path written by the spawner is the same path the next instance
/// resolves against.
#[test]
fn a_spaced_override_reaches_the_next_instance_unchanged() {
    let dir = PathBuf::from(r"D:\PathMaster data\v2");
    let line = CommandLine::relaunch(StartTab::System, &Location::Override(dir.clone()));

    let parsed = Arguments::parse(CommandLine::split(&line.into_os_string()));

    assert_eq!(parsed.tab, Some(StartTab::System));
    assert_eq!(data_dir(&parsed), Some(dir.as_os_str()));
    assert!(parsed.unknown.is_empty());
    assert!(!parsed.help);
}

/// A Run that was not pointed anywhere carries no switch — the elevated
/// instance locates its own `data\` beside the executable, which is the same
/// binary and so the same directory (ADR-0002).
#[test]
fn a_default_run_relaunches_with_the_tab_alone() {
    for location in [
        Location::BesideExe(PathBuf::from(r"C:\Tools\PathMaster\data")),
        Location::OwnLocationUnknown,
    ] {
        let line = CommandLine::relaunch(StartTab::User, &location).into_os_string();
        assert_eq!(line, OsString::from("--tab user"), "{location:?}");
    }
}

/// A broken override crosses as the bare switch it was. Dropping it would land
/// the elevated instance in the default `data\` — writing where it was not
/// pointed, which §10 forbids outright — so the elevated instance inherits the
/// same Read-only Data instead.
#[test]
fn a_broken_override_still_crosses_the_boundary() {
    let line = CommandLine::relaunch(StartTab::User, &Location::BrokenOverride).into_os_string();

    assert_eq!(line, OsString::from("--tab user --data-dir"));
    let parsed = Arguments::parse(CommandLine::split(&line));
    assert_eq!(data_dir(&parsed), Some(OsStr::new("")));
}

/// Unknown arguments die at the boundary: the relaunch is built from parsed
/// state, so there is nowhere for one to ride along, and the elevated instance
/// is never owed a second dialog about a typo already dismissed.
#[test]
fn unknown_arguments_do_not_cross_the_boundary() {
    let parsed = parse(&["--elevated-write", "--tab", "system", "nonsense"]);
    assert_eq!(unknown(&parsed), ["--elevated-write", "nonsense"]);

    let line = CommandLine::relaunch(
        parsed.tab.unwrap(),
        &Location::BesideExe(PathBuf::from(r"C:\Tools\data")),
    )
    .into_os_string();

    assert!(Arguments::parse(CommandLine::split(&line))
        .unknown
        .is_empty());
}

/// The one quoting rule worth reading in the output rather than through a round
/// trip: a path ending in a backslash has that backslash doubled, because
/// otherwise it would escape the quote meant to close the argument. Everywhere
/// else backslashes stay exactly as many as they were.
#[test]
fn only_a_backslash_facing_a_quote_is_doubled() {
    let mut line = CommandLine::default();
    line.push(OsStr::new(r"C:\two words\"));
    assert_eq!(line.into_os_string(), OsString::from(r#""C:\two words\\""#));

    let mut plain = CommandLine::default();
    plain.push(OsStr::new(r"C:\no-spaces\here"));
    assert_eq!(plain.into_os_string(), OsString::from(r"C:\no-spaces\here"));
}

/// Sanity on the splitter itself: it must model Windows, not just agree with
/// our writer. These are lines a human would type, read against what
/// `CommandLineToArgvW` documents.
#[test]
fn the_splitter_reads_a_hand_typed_line_the_way_windows_does() {
    let cases: &[(&str, &[&str])] = &[
        (
            r#"--data-dir "C:\Program Files\PM""#,
            &["--data-dir", r"C:\Program Files\PM"],
        ),
        (r#"a\\b c"#, &[r"a\\b", "c"]),
        (r#""a b\\" c"#, &[r"a b\", "c"]),
        (r#"a"b c"d"#, &["ab cd"]),
        ("   spaced\tout   ", &["spaced", "out"]),
        ("", &[]),
    ];
    for (line, expected) in cases {
        let split = CommandLine::split(OsStr::new(line));
        let expected: Vec<OsString> = expected.iter().map(OsString::from).collect();
        assert_eq!(split, expected, "line: {line:?}");
    }
}

/// The `--data-dir` value the spawner writes is the *resolved* path, not the
/// user's spelling — which is what [`Location`] carries and why the relaunch
/// takes one rather than a string.
#[test]
fn the_line_carries_the_resolved_path_and_not_a_relative_one() {
    let line = CommandLine::relaunch(
        StartTab::User,
        &Location::Override(Path::new(r"C:\work\pm-data").to_path_buf()),
    )
    .into_os_string();

    assert!(
        line.to_string_lossy().contains(r"C:\work\pm-data"),
        "{line:?}"
    );
}
