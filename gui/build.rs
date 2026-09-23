// Embeds the exe icon and version info on Windows. The window icon is set separately at runtime
// from assets/icon.png.
fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=assets/cloaker.ico");
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows") {
        winresource::WindowsResource::new()
            .set_icon("assets/cloaker.ico")
            // both default to the package name, cloaker_gui
            .set("ProductName", "Cloaker")
            .set("FileDescription", "Cloaker")
            .compile()
            .expect("failed to embed the Windows resources");
    }
}
