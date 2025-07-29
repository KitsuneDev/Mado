use std::{env, fs, path::PathBuf};

use shadow_rs::ShadowBuilder;

use wry_cmd;

fn main() {
    let _shadow = ShadowBuilder::builder().build().unwrap();

    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let src_dir = manifest_dir.join("src");
    let mado_dir = manifest_dir.join("../mado/src");
    let docs_file = manifest_dir.join("docs/commands");
    let _ = fs::create_dir_all(&docs_file);
    wry_cmd::generate_docs(&[src_dir, mado_dir], &docs_file)
        .expect("failed to generate command docs");

    // This should always be true, but anyways...
    if cfg!(target_os = "windows") {
        let mut res = winres::WindowsResource::new();
        //res.set_icon("path/to/your/icon.ico"); // Optional: set a custom icon
        res.set_version_info(winres::VersionInfo::FILETYPE, 2); // Set FILETYPE to DLL (2)
        // Other metadata can be pulled from Cargo.toml or set explicitly
        // TODO: Check https://docs.rs/winres/latest/winres/struct.WindowsResource.html
        res.set("ProductName", "Shigure");
        res.set("FileDescription", "A Mado Host for Rainmeter");
        res.set("OriginalFilename", "shigure.dll");
        res.set("CompanyName", "Kitsune");
        //res.set("Comments", "")
        res.compile().expect("Failed to compile Windows resources.");
    }
}
