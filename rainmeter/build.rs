use std::env;
use std::path::Path;

fn main() {
    // Re-run this build script if anything in libs/ changes
    println!("cargo:rerun-if-changed=sdk");

    // e.g. "x86_64-pc-windows-msvc" or "i686-pc-windows-msvc"
    let target = env::var("TARGET").unwrap();
    // Decide folder based on architecture substring
    let arch_dir = if target.contains("x86_64") {
        "x64"
    } else if target.contains("i686") {
        "x86"
    } else {
        panic!("Unsupported TARGET for Rainmeter crate: {}", target);
    };

    // Tell rustc where to find the .lib
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR not set");
    let lib_path = Path::new(&manifest_dir)
        .join("sdk")
        .join("API")
        .join(arch_dir);
    println!("cargo:rustc-link-search=native={}", lib_path.display());

    // Link the import‑library.  Since Rainmeter.lib is an import lib for a DLL,
    // we use `dylib` here.  If it were a truly static library, use `static` instead.
    println!("cargo:rustc-link-lib=dylib=Rainmeter");
}
