//! Windows only: embed the app icon and version info (VERSIONINFO) into `deckcraft.exe`, so it
//! shows in Explorer, the taskbar, the Start menu and Alt-Tab.
//!
//! On every other target this does nothing. A missing resource compiler is a warning, so a
//! cross-compile from macOS or Linux still links, unless `DECKCRAFT_REQUIRE_WINRES=1` turns it
//! into an error (for release builds).

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=../../nexus/assets/nexus-slides.ico");
    println!("cargo:rerun-if-env-changed=DECKCRAFT_REQUIRE_WINRES");
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() != Ok("windows") {
        return;
    }
    let mut res = winresource::WindowsResource::new();
    res.set_icon("../../nexus/assets/nexus-slides.ico")
        .set("ProductName", "Nexus Slides")
        .set("FileDescription", "Nexus Slides presentations")
        .set("CompanyName", "Sonnytech")
        .set("LegalCopyright", "Nexus Slides (c) 2026 Sonnytech; based on DeckCraft by the ArtCraft team. MIT OR Apache-2.0.")
        .set("OriginalFilename", "nexus-slides.exe")
        .set("InternalName", "deckcraft");
    if let Err(e) = res.compile() {
        if std::env::var_os("DECKCRAFT_REQUIRE_WINRES").is_some() {
            eprintln!("embedding Windows resources failed: {e}");
            std::process::exit(1);
        }
        println!("cargo:warning=deckcraft.exe built without icon/version resources: {e}");
    }
}
