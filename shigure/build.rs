use shadow_rs::ShadowBuilder;

fn main() {
    let shadow = ShadowBuilder::builder().build().unwrap();
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
