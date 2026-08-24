//! Build inputs the binary cannot carry as Rust source: the application
//! manifest, the exe's icon and VERSIONINFO, and the Catalogue's compiled
//! `.mo` files.

use std::fmt::Write as _;
use std::path::{Path, PathBuf};
use std::process::Command;

use polib::po_file::POParseOptions;

fn main() {
    embed_manifest();
    embed_resources();
    compile_catalogues();
}

/// Embeds `app.manifest` into the exe via the MSVC linker, with no extra crate
/// dependency. The linker still contributes its own default trustInfo
/// (asInvoker), so `app.manifest` deliberately does not declare one — two
/// trustInfo blocks would be a merge conflict.
fn embed_manifest() {
    let manifest = manifest_dir().join("app.manifest");
    println!("cargo:rerun-if-changed=app.manifest");
    println!("cargo:rustc-link-arg-bins=/MANIFEST:EMBED");
    println!(
        "cargo:rustc-link-arg-bins=/MANIFESTINPUT:{}",
        manifest.display()
    );
}

/// Embeds `resources/app.rc` — the icon and the VERSIONINFO — by compiling it
/// to a `.res` and handing that to the linker as an input file (spec §12, §16).
///
/// `/MANIFEST:EMBED` works because the linker generates that one resource
/// itself; there is no equivalent switch for an icon. An icon is an
/// `RT_GROUP_ICON` directory plus one `RT_ICON` per image in the PE resource
/// directory, and `include_bytes!` cannot produce that — it puts bytes in a
/// data section, which the shell never reads. A compiled `.res` is the only
/// route, and it travels down the same `rustc-link-arg-bins` channel the
/// manifest already uses.
///
/// **This governs Explorer, Alt+Tab from the file, and pinned shortcuts — not
/// the running window.** wxMSW does not adopt the exe's icon resource for a
/// frame, whose icon is separate and unset until [`crate::ui`] sets it from
/// the SVG (research/04 §4.2 measured both halves).
fn embed_resources() {
    let resources = manifest_dir().join("resources");
    println!("cargo:rerun-if-changed=resources/app.rc");
    println!("cargo:rerun-if-changed=resources/app.ico");

    let res = out_dir().join("app.res");
    let compiler = resource_compiler();
    let status = Command::new(&compiler)
        // The `.rc` names its icon relatively, so the include path is what
        // makes it findable from wherever cargo happens to run this.
        .arg("/I")
        .arg(&resources)
        .arg("/FO")
        .arg(&res)
        .arg(resources.join("app.rc"))
        .status()
        .unwrap_or_else(|e| panic!("{} could not be run: {e}", compiler.display()));
    assert!(status.success(), "{} failed: {status}", compiler.display());

    println!("cargo:rustc-link-arg-bins={}", res.display());
}

/// `llvm-rc`, preferred from the LLVM installation `LIBCLANG_PATH` already
/// points at.
///
/// The SDK's `rc.exe` would do the same job, but it is not on `PATH` and needs
/// registry discovery — the annoying part that `winresource` and
/// `embed-resource` exist to solve. `llvm-rc` sits beside the `libclang.dll`
/// this build hard-requires for bindgen, so preferring it adds **no dependency
/// that was not already mandatory** — and on the release runner, where the SDK
/// `bin` is not on `PATH`, it is the only one of the two that is findable at
/// all.
fn resource_compiler() -> PathBuf {
    println!("cargo:rerun-if-env-changed=LIBCLANG_PATH");
    std::env::var_os("LIBCLANG_PATH")
        .map(|libclang| Path::new(&libclang).join("llvm-rc.exe"))
        .filter(|candidate| candidate.is_file())
        // Not an error: a machine with LLVM on `PATH` and no `LIBCLANG_PATH`
        // set is a machine where bindgen has already found its own way.
        .unwrap_or_else(|| PathBuf::from("llvm-rc"))
}

/// Compiles every `i18n/<code>.po` into `OUT_DIR/<code>.mo` and writes the table
/// `catalog.rs` includes (spec §11). Everything a new language needs from the
/// build is this enumeration: drop `xx.po` in and the `.mo`, the table and
/// `available_translations` follow. (Naming the language to the rest of the
/// application is a separate hand edit — see `pathmaster_core::language`.)
///
/// `polib` writes what it is given, so the two exclusions `msgfmt` performs are
/// performed here: an untranslated message would answer with an *empty string*
/// where a miss should have fallen back to the English msgid, and a fuzzy one —
/// gettext's "guessed after the source changed" — must read as missing, which is
/// what the completeness gate assumes when it reads the `.po` rather than the `.mo`.
fn compile_catalogues() {
    let i18n = manifest_dir().join("i18n");
    let out_dir = out_dir();
    println!("cargo:rerun-if-changed=i18n");

    let mut codes: Vec<String> = Vec::new();
    for entry in std::fs::read_dir(&i18n).expect("the i18n directory") {
        let path = entry.expect("an i18n directory entry").path();
        if path.extension().and_then(|e| e.to_str()) != Some("po") {
            continue;
        }
        let code = path
            .file_stem()
            .and_then(|s| s.to_str())
            .expect("a language code")
            .to_owned();
        println!("cargo:rerun-if-changed=i18n/{code}.po");
        compile_catalogue(&path, &out_dir.join(format!("{code}.mo")));
        codes.push(code);
    }
    assert!(
        !codes.is_empty(),
        "no catalogue found in {}",
        i18n.display()
    );
    codes.sort();

    let mut table = String::from(
        "// Generated by build.rs from i18n/*.po — one row per language.\n\
         static CATALOGUES: &[(&str, &[u8])] = &[\n",
    );
    for code in &codes {
        writeln!(
            table,
            "    ({code:?}, include_bytes!(concat!(env!(\"OUT_DIR\"), \"/{code}.mo\"))),"
        )
        .expect("writing to a String");
    }
    table.push_str("];\n");
    std::fs::write(out_dir.join("catalogues.rs"), table).expect("writing the catalogue table");
}

fn compile_catalogue(po: &Path, mo: &Path) {
    let options = POParseOptions {
        translated_only: true,
        ..POParseOptions::new()
    };
    let mut catalog = polib::po_file::parse_with_option(po, &options)
        .unwrap_or_else(|e| panic!("{} does not parse: {e}", po.display()));

    let fuzzy: Vec<(String, Option<String>)> = catalog
        .messages()
        .filter(|message| message.is_fuzzy())
        .map(|message| {
            (
                message.msgid().to_owned(),
                message.msgid_plural().ok().map(str::to_owned),
            )
        })
        .collect();
    for (msgid, plural) in fuzzy {
        catalog.delete_message(None, &msgid, plural.as_deref());
    }

    polib::mo_file::write(&catalog, mo)
        .unwrap_or_else(|e| panic!("{} could not be written: {e}", mo.display()));
}

fn manifest_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn out_dir() -> PathBuf {
    PathBuf::from(std::env::var_os("OUT_DIR").expect("OUT_DIR is set by cargo"))
}
