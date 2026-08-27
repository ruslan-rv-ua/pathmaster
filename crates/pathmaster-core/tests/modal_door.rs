//! The modal door's build gate (spec §11, ADR-0011): `show_modal` may appear
//! in the binary crate only inside the door module.
//!
//! Every modal dialog runs a nested event loop, and the diagnostic Timer ticks
//! inside it — the hazard ADR-0011 closes by routing every dialog through one
//! door that counts modal depth. A dialog opened around the door would be a
//! dialog the Timer gate cannot see, so the rule is enforced where the User
//! Guide's heading parity will be: by a source scan that fails the build.
//!
//! It lives here, in the crate that links no wxWidgets, for the reason
//! `versioninfo.rs` gives: this is a test-time path, not a dependency edge,
//! and making it one of the binary's tests would cost ADR-0009 the claim that
//! the msgid smoke test remains the only test linking wxWidgets.
//!
//! The scan is deliberately a token scan, comments included: prose about
//! `show_modal` outside the door is prose steering the next reader around it,
//! and the door's own name — `door::show` — is the one to write instead.

use std::path::{Path, PathBuf};

/// The binary crate's sources, reached the way `catalogue.rs` reaches its
/// `i18n` directory — resolved from this manifest at compile time, so the scan
/// cannot be defeated by being run from a different directory.
fn binary_src() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../pathmaster/src")
}

/// Every `.rs` file under `dir`, recursively — the whole binary crate, so a
/// file added tomorrow is scanned tomorrow, not remembered here.
fn rust_sources(dir: &Path, sources: &mut Vec<PathBuf>) {
    for entry in std::fs::read_dir(dir).expect("the binary crate's src directory") {
        let path = entry.expect("a readable directory entry").path();
        if path.is_dir() {
            rust_sources(&path, sources);
        } else if path.extension().is_some_and(|extension| extension == "rs") {
            sources.push(path);
        }
    }
}

#[test]
fn show_modal_appears_only_inside_the_door() {
    let mut sources = Vec::new();
    rust_sources(&binary_src(), &mut sources);
    assert!(
        !sources.is_empty(),
        "the scan found no sources — the path it walks has moved"
    );

    let mut door_scanned = false;
    for path in sources {
        let source = std::fs::read_to_string(&path).expect("a readable source file");
        if path.ends_with("ui/door.rs") || path.ends_with("ui\\door.rs") {
            // The door itself must make the call: a door module the token has
            // left is a door the dialogs no longer pass through, and this scan
            // silently scanning nothing would be its own way to go stale.
            assert!(
                source.contains("show_modal"),
                "ui/door.rs no longer calls show_modal — the door is not a door"
            );
            door_scanned = true;
            continue;
        }
        assert!(
            !source.contains("show_modal"),
            "show_modal outside the door module, in {} — route the dialog \
             through door::show so the Timer gate can see it (ADR-0011)",
            path.display()
        );
    }
    assert!(
        door_scanned,
        "ui/door.rs was not scanned — the door module has moved or is missing"
    );
}
