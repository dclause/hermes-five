use std::path::Path;

fn main() {
    // Warning: build.rs is not published to crates.io.

    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=src/mod.rs");

    println!("cargo:rustc-check-cfg=cfg(build_from_git)");

    // Sometimes on Windows the git checkout does not correctly wire up the
    // symlink from hermes-macros-internals/src to hermes-macros/src/internals.
    // When this happens we'll just build based on relative paths within the git
    // repo.
    let mod_behind_symlink = Path::new("src/mod.rs");
    if !mod_behind_symlink.exists() {
        println!("cargo:rustc-cfg=build_from_git");
    }
}
