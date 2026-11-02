#[path = "src/commands/manifest.rs"]
mod manifest;
fn main() {
    // Desktop plugin tests link TaskDialogIndirect too. Integration-test executables need
    // the same Common Controls v6 activation as the bundled Windows executable.
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows")
        && std::env::var("CARGO_CFG_TARGET_ENV").as_deref() == Ok("msvc")
    {
        println!("cargo:rustc-link-arg-tests=/MANIFEST:EMBED");
        println!("cargo:rustc-link-arg-tests=/MANIFESTDEPENDENCY:type='win32' name='Microsoft.Windows.Common-Controls' version='6.0.0.0' processorArchitecture='*' publicKeyToken='6595b64144ccf1df' language='*'");
    }
    tauri_build::try_build(
        tauri_build::Attributes::new()
            .app_manifest(tauri_build::AppManifest::new().commands(manifest::APP_COMMANDS)),
    )
    .expect("Could not build application ACL");
}
