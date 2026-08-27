//! Expansion Mode's rendering rule at the crate boundary (v0.2.0 spec §5,
//! §13 item 8; ticket impl 04).
//!
//! One question: what does the list show for this Entry right now? Raw mode
//! answers with the stored text, untouched. Expanded mode answers with
//! Normalisation's own expansion — `ExpandEnvironmentStringsW`'s reading of
//! the process environment — and **nothing else**: no quote stripping, no
//! trailing-`\` trimming, no case fold, because those change *what text
//! exists* and this is a rendering, not a comparison key.

use pathmaster_core::expansion::Mode;
use pathmaster_core::filtered::matches;
use pathmaster_core::normalize::Environment;

/// A fixed environment, looked up case-insensitively as Windows' own is.
struct Env(&'static [(&'static str, &'static str)]);

impl Environment for Env {
    fn lookup(&self, name: &str) -> Option<String> {
        self.0
            .iter()
            .find(|(key, _)| key.eq_ignore_ascii_case(name))
            .map(|(_, value)| (*value).to_string())
    }
}

const ENV: Env = Env(&[("SystemRoot", r"C:\Windows"), ("JAVA_HOME", r"C:\jdk21")]);

fn rendered(mode: Mode, raw: &str) -> String {
    mode.render(raw, &ENV).into_owned()
}

// ---------------------------------------------------------------- the modes

#[test]
fn a_run_starts_raw() {
    // Per-Run, default raw, nothing persists: the mode a `Default` gives is
    // the mode every Run opens in (v0.2.0 §5).
    assert_eq!(Mode::default(), Mode::Raw);
    assert!(!Mode::default().expanded());
}

#[test]
fn the_toggle_flips_and_flips_back() {
    assert_eq!(Mode::Raw.toggled(), Mode::Expanded);
    assert_eq!(Mode::Expanded.toggled(), Mode::Raw);
    assert_eq!(Mode::Raw.toggled().toggled(), Mode::Raw);
}

#[test]
fn only_the_expanded_mode_reads_as_expanded() {
    // What the `wxITEM_CHECK` item's mark says, answered here so the menu and
    // the rendering cannot disagree.
    assert!(Mode::Expanded.expanded());
    assert!(!Mode::Raw.expanded());
}

// --------------------------------------------------------------- raw mode

#[test]
fn raw_mode_shows_the_stored_text_untouched() {
    assert_eq!(rendered(Mode::Raw, r"%JAVA_HOME%\bin"), r"%JAVA_HOME%\bin");
    assert_eq!(
        rendered(Mode::Raw, r"C:/Program Files/"),
        r"C:/Program Files/"
    );
    assert_eq!(
        rendered(Mode::Raw, r#""C:\Program Files""#),
        r#""C:\Program Files""#
    );
}

#[test]
fn raw_mode_borrows_rather_than_copying() {
    // Every rebuild renders every visible row, and the default mode has
    // nothing to do — so it allocates nothing.
    let raw = r"%JAVA_HOME%\bin";
    assert!(matches!(
        Mode::Raw.render(raw, &ENV),
        std::borrow::Cow::Borrowed(_)
    ));
}

// ----------------------------------------------------------- expanded mode

#[test]
fn expanded_mode_resolves_a_defined_reference() {
    assert_eq!(
        rendered(Mode::Expanded, r"%JAVA_HOME%\bin"),
        r"C:\jdk21\bin"
    );
    // The lookup ignores case, as `GetEnvironmentVariableW`'s own does.
    assert_eq!(rendered(Mode::Expanded, r"%java_home%"), r"C:\jdk21");
    assert_eq!(
        rendered(Mode::Expanded, r"%systemroot%\system32"),
        r"C:\Windows\system32"
    );
}

#[test]
fn an_undefined_reference_stays_literal_in_place() {
    // No new Issue type and no inline marker: the Status column's natural
    // `Missing` already answers "why" (v0.2.0 §5).
    assert_eq!(rendered(Mode::Expanded, r"%NOPE%\bin"), r"%NOPE%\bin");
    assert_eq!(
        rendered(Mode::Expanded, r"%NOPE%\bin;%JAVA_HOME%\bin"),
        r"%NOPE%\bin;C:\jdk21\bin"
    );
}

#[test]
fn expansion_is_the_reading_normalisation_already_makes() {
    // What is shown can never disagree with what is diagnosed, so the display
    // takes the same pass and not a second one: `%%` resolves nothing, an
    // unterminated `%` is ordinary text, and a value carrying a reference of
    // its own is not expanded again.
    assert_eq!(rendered(Mode::Expanded, "%%"), "%%");
    assert_eq!(rendered(Mode::Expanded, r"50%"), r"50%");
    // A failed reference gives its closing `%` back to the scan, which is what
    // opens the one that succeeds.
    assert_eq!(
        rendered(Mode::Expanded, r"%NOPE%SystemRoot%"),
        r"%NOPEC:\Windows"
    );
}

#[test]
fn a_rendering_is_not_a_comparison_key() {
    // Quotes, slash direction and the trailing separator all survive: the
    // Normalisation steps that fold them answer "are these the same path?",
    // which is not the question a list asks.
    assert_eq!(
        rendered(Mode::Expanded, r#""%JAVA_HOME%\bin""#),
        r#""C:\jdk21\bin""#
    );
    assert_eq!(rendered(Mode::Expanded, r"%SystemRoot%/"), r"C:\Windows/");
}

#[test]
fn an_entry_with_nothing_to_expand_renders_alike_in_both_modes() {
    for mode in [Mode::Raw, Mode::Expanded] {
        assert_eq!(rendered(mode, r"C:\tools"), r"C:\tools");
    }
}

// ------------------------------------------- the coupling Search is read by

#[test]
fn the_two_modes_are_different_haystacks() {
    // Search matches the currently displayed rendering, so the toggle changes
    // membership under a Filtered View — paid deliberately (v0.2.0 §3, §5).
    let raw = r"%JAVA_HOME%\bin";
    assert!(matches(&Mode::Raw.render(raw, &ENV), "java_home"));
    assert!(!matches(&Mode::Raw.render(raw, &ENV), "jdk21"));
    assert!(matches(&Mode::Expanded.render(raw, &ENV), "jdk21"));
    assert!(!matches(&Mode::Expanded.render(raw, &ENV), "java_home"));
}
