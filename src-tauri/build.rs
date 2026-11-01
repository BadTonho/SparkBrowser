#[path = "src/commands/manifest.rs"]
mod manifest;
fn main() {
    tauri_build::try_build(
        tauri_build::Attributes::new()
            .app_manifest(tauri_build::AppManifest::new().commands(manifest::APP_COMMANDS)),
    )
    .expect("Could not build application ACL");
}
