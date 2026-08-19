// Embeds app.manifest into the exe via the MSVC linker, with no extra crate dependency.
// The linker still contributes its own default trustInfo (asInvoker), so app.manifest
// deliberately does not declare one - two trustInfo blocks would be a merge conflict.
fn main() {
    let manifest = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("app.manifest");
    println!("cargo:rerun-if-changed=app.manifest");
    println!("cargo:rustc-link-arg-bins=/MANIFEST:EMBED");
    println!(
        "cargo:rustc-link-arg-bins=/MANIFESTINPUT:{}",
        manifest.display()
    );
}
