//! What version this build is — and the gate that keeps the exe from saying
//! something else (spec §16).
//!
//! An unsigned binary's identity is its `VERSIONINFO` and nothing more, so the
//! version appears in three places that can disagree: `Cargo.toml`, the
//! `resources/app.rc` the linker compiles in, and the git tag a release is cut
//! from. The release workflow checks all three before it builds. The test
//! below checks the first two on **every** `cargo test`, which is where drift
//! is actually introduced — a version bump is a `Cargo.toml` edit, and the
//! `.rc` is the file it is easy to forget.

/// This build's version, as Cargo knows it — which is the workspace's, since
/// the crate takes `version.workspace = true`.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg(test)]
mod tests {
    use super::VERSION;

    /// The resource script, read at compile time: the test needs no filesystem
    /// and cannot be defeated by being run from a different directory.
    const RC: &str = include_str!("../resources/app.rc");

    /// The `.rc`'s statements with their `//` comments removed, so prose about
    /// a field cannot be mistaken for the field. (No string in the file
    /// contains `//`; a path-valued one some day would need this to be
    /// cleverer, and would fail loudly rather than quietly.)
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
    fn the_versioninfo_carries_the_crate_version() {
        // Four fields, because Windows reads two of them and winget reads the
        // other two: the binary FILEVERSION/PRODUCTVERSION are what a version
        // comparison uses, and the strings are what Explorer's Properties tab
        // and the release workflow's gate read.
        let [major, minor, patch] =
            <[&str; 3]>::try_from(VERSION.split('.').collect::<Vec<&str>>())
                .expect("the crate version is major.minor.patch");
        let quad = format!("{major},{minor},{patch},0");

        assert_eq!(binary_version("FILEVERSION"), quad);
        assert_eq!(binary_version("PRODUCTVERSION"), quad);
        assert_eq!(string_value("FileVersion"), VERSION);
        assert_eq!(string_value("ProductVersion"), VERSION);
    }

    #[test]
    fn the_versioninfo_names_the_binary_cargo_actually_builds() {
        // `OriginalFilename` is how a renamed copy is traced back to what it
        // was, and both package managers rename this exe: scoop's `#/` URL
        // fragment and winget's `Commands` each put a different name on disk.
        // The name it should carry is therefore the one cargo links, not a
        // string typed twice.
        assert_eq!(string_value("InternalName"), env!("CARGO_BIN_NAME"));
        assert_eq!(
            string_value("OriginalFilename"),
            format!("{}.exe", env!("CARGO_BIN_NAME"))
        );
    }

    #[test]
    fn the_versioninfo_carries_the_identity_the_package_managers_were_built_from() {
        // `CompanyName` is the publisher half of the winget PackageIdentifier
        // `RuslanIskov.PathMaster` (spec §16), and for an unsigned binary the
        // VERSIONINFO is the only place that claim is made at all. Both are
        // asserted here so that changing one is a failing test rather than a
        // quiet disagreement with a manifest in another repository.
        assert_eq!(string_value("CompanyName"), "Ruslan Iskov");
        assert_eq!(string_value("ProductName"), "PathMaster");
        assert!(
            string_value("LegalCopyright").contains("MIT"),
            "the licence is MIT (spec §16) and the copyright line says so"
        );
    }
}
