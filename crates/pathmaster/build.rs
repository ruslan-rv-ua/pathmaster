//! Build inputs the binary cannot carry as Rust source: the application
//! manifest, the exe's icon and VERSIONINFO, the Catalogue's compiled `.mo`
//! files, and the User Guide's pages.

use std::fmt::Write as _;
use std::path::{Path, PathBuf};
use std::process::Command;

use polib::po_file::POParseOptions;

fn main() {
    embed_manifest();
    embed_resources();
    // The languages are enumerated once, by the catalogues, and the User Guide
    // is held to the same list: a language that ships a catalogue and no page
    // would be one whose F1 opened someone else's guide (v0.2.0 §9).
    let languages = compile_catalogues();
    compile_help(&languages);
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
fn compile_catalogues() -> Vec<String> {
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
    codes
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

/// Converts `docs/help/<code>.md` into `OUT_DIR/help-<code>.html` for every
/// language that ships, and writes the table `help.rs` includes — the `.mo`
/// mechanism mirrored, because the page has exactly the Catalogue's problem:
/// it is text that has to travel inside a single executable (v0.2.0 §9,
/// NFR-portable).
///
/// The documents live at the repository root rather than beside this crate,
/// and reaching up for them is deliberate: the failing rung of §9's ladder
/// points a browser at `…/blob/v{version}/docs/help/<code>.md`, so where they
/// sit is part of the contract — and the heading-parity gate in
/// `pathmaster-core` reads the same two files.
fn compile_help(languages: &[String]) {
    let help = manifest_dir().join(HELP_DIR);
    let out_dir = out_dir();
    println!("cargo:rerun-if-changed={HELP_DIR}");

    let mut table = String::from(
        "// Generated by build.rs from docs/help/*.md — one row per language.\n\
         static HELP_PAGES: &[(&str, &[u8])] = &[\n",
    );
    for code in languages {
        let source = help.join(format!("{code}.md"));
        println!("cargo:rerun-if-changed={HELP_DIR}/{code}.md");
        let markdown = std::fs::read_to_string(&source).unwrap_or_else(|e| {
            panic!(
                "{} could not be read: {e} — every language that ships a catalogue \
                 ships a User Guide (v0.2.0 §9)",
                source.display()
            )
        });
        std::fs::write(
            out_dir.join(format!("help-{code}.html")),
            page(&markdown, code),
        )
        .expect("writing a User Guide page");
        writeln!(
            table,
            "    ({code:?}, include_bytes!(concat!(env!(\"OUT_DIR\"), \"/help-{code}.html\"))),"
        )
        .expect("writing to a String");
    }
    table.push_str("];\n");
    std::fs::write(out_dir.join("help_pages.rs"), table).expect("writing the User Guide table");
}

/// The whole page: the document rendered, inside the shell §9 fixes.
///
/// **It sets no colours.** `color-scheme: light dark` is the HTML equivalent of
/// the application's own rule — a page with no stylesheet at all is painted
/// black-on-white whatever the system theme says, which would not satisfy it —
/// and everything else here is layout. `lang` is load-bearing rather than
/// tidiness: without it a screen reader may read the Ukrainian guide in an
/// English voice.
fn page(markdown: &str, language_code: &str) -> String {
    let mut body = String::new();
    // Tables are the one extension the content needs — §9's contract includes
    // the full keyboard table, and CommonMark alone has no table.
    let parser = pulldown_cmark::Parser::new_ext(markdown, pulldown_cmark::Options::ENABLE_TABLES);
    pulldown_cmark::html::push_html(&mut body, parser);

    format!(
        "<!doctype html>\n\
         <html lang=\"{language_code}\">\n\
         <head>\n\
         <meta charset=\"utf-8\">\n\
         <meta name=\"viewport\" content=\"width=device-width, initial-scale=1\">\n\
         <title>{title}</title>\n\
         <style>\n{STYLE}</style>\n\
         </head>\n\
         <body>\n\
         {body}</body>\n\
         </html>\n",
        title = escaped(&title(markdown)),
    )
}

/// Layout, and the one line that is not layout.
///
/// `color-scheme` is what makes the page follow the reader's theme instead of
/// being painted black-on-white; every other rule here is width, spacing and
/// the system font. **No colour is named**, deliberately and to the letter —
/// the application sets none either, which is how High Contrast simply works.
const STYLE: &str = "\
:root { color-scheme: light dark; }
body { max-width: 44rem; margin: 0 auto; padding: 1rem; font-family: system-ui, sans-serif; line-height: 1.6; }
table { border-collapse: collapse; }
th, td { text-align: left; vertical-align: top; padding: 0.2rem 1.5rem 0.2rem 0; }
pre { overflow-x: auto; }
";

/// The page's `<title>` — the first thing a screen reader speaks when the page
/// loads, so it says which build's guide this is.
///
/// It is the document's own level-one heading with this build's version spliced
/// in after the product name: "PathMaster 0.2.0 — User Guide", and the
/// Ukrainian document's own words in the Ukrainian page. Composed from the
/// heading rather than written out here, because that is what keeps the title
/// translated without a second place to keep in step — and the shape is
/// asserted, so a document that renamed its own title fails the build rather
/// than shipping a page that announces itself as something else.
fn title(markdown: &str) -> String {
    const PRODUCT: &str = "PathMaster";

    // The document's **opening** line rather than the first `# ` anywhere:
    // the heading-parity gate already refuses a guide that does not open on
    // one title, and reading only that line is what keeps this from mistaking
    // a `#` inside the "Command line" subsection's fenced example for a
    // heading without repeating that gate's fence tracking here.
    let heading = markdown
        .lines()
        .find(|line| !line.trim().is_empty())
        .and_then(|line| line.strip_prefix("# "))
        .expect("a User Guide opens on a level-one heading")
        .trim();
    let rest = heading
        .strip_prefix(PRODUCT)
        .unwrap_or_else(|| panic!("a User Guide's title starts with {PRODUCT:?}: {heading:?}"));
    // Read from the environment rather than `env!`-ed: cargo guarantees this
    // variable to a build script it *runs*, which is the moment the version
    // has to be right.
    let version = std::env::var("CARGO_PKG_VERSION").expect("CARGO_PKG_VERSION is set by cargo");
    format!("{PRODUCT} {version}{rest}")
}

/// The `<title>` element's text is the one string on the page this file writes
/// rather than the renderer, so it is the one string this file has to escape:
/// `<` would close the element early, and `&` would open an entity reference
/// that swallowed what followed. Everything else came out of an HTML renderer
/// already escaped.
fn escaped(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

/// Where the User Guide's source documents live, relative to this crate — the
/// path the online fallback and the heading-parity gate both name.
const HELP_DIR: &str = "../../docs/help";

fn manifest_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn out_dir() -> PathBuf {
    PathBuf::from(std::env::var_os("OUT_DIR").expect("OUT_DIR is set by cargo"))
}
