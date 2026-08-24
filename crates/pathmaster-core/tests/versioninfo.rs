//! The exe's identity, gated against the version Cargo carries (spec §16).
//!
//! An unsigned binary's `VERSIONINFO` is the whole of what it says about
//! itself, and the version in it appears in three places that can disagree:
//! `Cargo.toml`, the `resources/app.rc` the linker compiles in, and the git tag
//! a release is cut from. The release workflow checks all three before it
//! builds. This checks the first two on **every** `cargo test`, which is where
//! drift is actually introduced — a version bump is a `Cargo.toml` edit, and
//! the `.rc` is the file it is easy to forget.
//!
//! It lives here, in the crate that links no wxWidgets, for the reason
//! `catalogue.rs` gives for reaching into `../pathmaster/i18n`: this is a
//! test-time path, not a dependency edge. The check is pure text over two files
//! read at compile time, so making it one of the binary's tests would tax it
//! with a wxWidgets link for nothing — and would cost ADR-0009 the claim that
//! the msgid smoke test "remains the only test that links wxWidgets"
//! (ADR-0007, ADR-0009).
//!
//! The version compared against is **this** crate's, which is the binary's
//! because both take `version.workspace = true` — asserted below rather than
//! assumed, since that inheritance is the whole reason one version can stand
//! for the other.

/// The resource script and the binary's manifest, read at compile time: the
/// tests need no filesystem and cannot be defeated by being run from a
/// different directory.
const RC: &str = include_str!("../../pathmaster/resources/app.rc");
const BIN_MANIFEST: &str = include_str!("../../pathmaster/Cargo.toml");

/// The workspace version, which every crate here inherits.
const VERSION: &str = env!("CARGO_PKG_VERSION");

/// The `.rc`'s statements with their `//` comments removed, so prose about a
/// field cannot be mistaken for the field. (No string in the file contains
/// `//`; a path-valued one some day would need this to be cleverer, and would
/// fail loudly rather than quietly.)
fn statements() -> impl Iterator<Item = &'static str> {
    RC.lines()
        .map(|line| line.split("//").next().unwrap_or("").trim())
        .filter(|line| !line.is_empty())
}

/// The text of `VALUE "<field>", "<text>"`.
fn string_value(field: &str) -> &'static str {
    let prefix = format!("VALUE \"{field}\"");
    let statement = statements()
        .find(|statement| statement.starts_with(&prefix))
        .unwrap_or_else(|| panic!("app.rc has no VALUE {field:?}"));
    statement
        .split('"')
        .nth(3)
        .unwrap_or_else(|| panic!("app.rc's {field:?} has no quoted value: {statement:?}"))
}

/// The comma-separated numbers of a bare `FILEVERSION`/`PRODUCTVERSION`
/// statement, with the spacing taken out so the comparison is about the
/// numbers.
fn binary_version(statement_name: &str) -> String {
    let statement = statements()
        .find(|statement| statement.starts_with(statement_name))
        .unwrap_or_else(|| panic!("app.rc has no {statement_name} statement"));
    statement[statement_name.len()..]
        .split(',')
        .map(str::trim)
        .collect::<Vec<&str>>()
        .join(",")
}

#[test]
fn the_binary_takes_the_workspace_version_this_crate_takes() {
    // What makes every assertion below meaningful: the version measured here
    // is this crate's, and it stands for the executable's only because the
    // binary inherits the same one. An independent version in that manifest
    // would leave these tests passing while the exe carried something else.
    assert!(
        statements_of(BIN_MANIFEST).any(|line| line == "version.workspace = true"),
        "crates/pathmaster/Cargo.toml must inherit the workspace version"
    );
}

#[test]
fn the_versioninfo_carries_the_crate_version() {
    // Four fields, because Windows reads two of them and winget reads the
    // other two: the binary FILEVERSION/PRODUCTVERSION are what a version
    // comparison uses, and the strings are what Explorer's Properties tab and
    // the release workflow's gate read.
    let [major, minor, patch] = <[&str; 3]>::try_from(VERSION.split('.').collect::<Vec<&str>>())
        .expect("the workspace version is major.minor.patch");
    let quad = format!("{major},{minor},{patch},0");

    assert_eq!(binary_version("FILEVERSION"), quad);
    assert_eq!(binary_version("PRODUCTVERSION"), quad);
    assert_eq!(string_value("FileVersion"), VERSION);
    assert_eq!(string_value("ProductVersion"), VERSION);
}

#[test]
fn the_versioninfo_names_the_binary_cargo_actually_builds() {
    // `OriginalFilename` is how a renamed copy is traced back to what it was,
    // and both package managers rename this exe: scoop's `#/` URL fragment and
    // winget's `Commands` each put a different name on disk. The name it
    // should carry is therefore the one cargo links — read out of the same
    // manifest that declares it, not typed a second time here.
    assert_eq!(string_value("InternalName"), bin_name());
    assert_eq!(
        string_value("OriginalFilename"),
        format!("{}.exe", bin_name())
    );
}

#[test]
fn the_versioninfo_carries_the_identity_the_package_managers_were_built_from() {
    // `CompanyName` is the publisher half of the winget PackageIdentifier
    // `RuslanIskov.PathMaster` (spec §16), and for an unsigned binary the
    // VERSIONINFO is the only place that claim is made at all. Both are
    // asserted here so that changing one is a failing test rather than a quiet
    // disagreement with a manifest in another repository.
    assert_eq!(string_value("CompanyName"), "Ruslan Iskov");
    assert_eq!(string_value("ProductName"), "PathMaster");
    assert!(
        string_value("LegalCopyright").contains("MIT"),
        "the licence is MIT (spec §16) and the copyright line says so"
    );
}

/// A TOML file's lines, comments and blanks removed — the same reading
/// [`statements`] gives the `.rc`, for the manifest's own `#` comment syntax.
fn statements_of(toml: &'static str) -> impl Iterator<Item = &'static str> {
    toml.lines()
        .map(|line| line.split('#').next().unwrap_or("").trim())
        .filter(|line| !line.is_empty())
}

/// The `[[bin]]` section's `name` — what cargo links the executable as, and so
/// what the `VERSIONINFO` has to call it.
///
/// Scoped to the section rather than counted from the top of the file: the
/// package's own `name` is a different string ("pathmaster", lower case), and a
/// positional read would compare against whichever of them the manifest
/// happened to list first.
fn bin_name() -> &'static str {
    statements_of(BIN_MANIFEST)
        .skip_while(|line| *line != "[[bin]]")
        .find_map(|line| line.strip_prefix("name = "))
        .and_then(|value| value.split('"').nth(1))
        .expect("crates/pathmaster/Cargo.toml declares a [[bin]] name")
}
