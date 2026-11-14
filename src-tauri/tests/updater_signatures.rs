#![cfg(windows)]

use std::{
    io::{Read, Write},
    net::TcpListener,
    process::Command,
    time::Duration,
};
use tauri_plugin_updater::UpdaterExt;

fn updater(
    public_key: &str,
) -> (
    tauri::App<tauri::test::MockRuntime>,
    tauri_plugin_updater::Updater,
) {
    spark_browser_lib::updates::initialize_crypto();
    let mut context = tauri::test::mock_context(tauri::test::noop_assets());
    context.config_mut().identifier = "org.spark.browser.updater-unit-tests".into();
    context.config_mut().plugins.0.insert("updater".into(), serde_json::json!({"pubkey":public_key,"requireSignedVersion":true,"endpoints":["https://example.invalid/updater.json"]}));
    let app = tauri::test::mock_builder()
        .plugin(tauri_plugin_updater::Builder::new().build())
        .build(context)
        .unwrap();
    let updater = app
        .updater_builder()
        .target("windows-x86_64")
        .build()
        .unwrap();
    (app, updater)
}

/// Real CLI signatures + the official plugin, without executing an installer or using production keys.
#[tokio::test]
async fn signed_packages_bind_the_version_and_reject_tampering_and_interrupted_downloads() {
    let folder = tempfile::tempdir().unwrap();
    let key = folder.path().join("fixture.key");
    let artifact = folder.path().join("fixture-setup.exe");
    std::fs::write(&artifact, b"artificial updater payload; never executed").unwrap();
    let cli = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .join("node_modules/@tauri-apps/cli/tauri.js");
    let result = Command::new("node")
        .arg(&cli)
        .args(["signer", "generate", "--ci", "--write-keys"])
        .arg(&key)
        .output()
        .unwrap();
    assert!(
        result.status.success(),
        "Test signing-key generation failed"
    );
    let result = Command::new("node")
        .arg(&cli)
        .args(["signer", "sign", "--private-key-path"])
        .arg(&key)
        .args(["--password", "", "--app-version", "1.2.0"])
        .arg(&artifact)
        .output()
        .unwrap();
    assert!(result.status.success(), "Test artifact signing failed");
    let public = std::fs::read_to_string(key.with_extension("key.pub")).unwrap();
    let signature = std::fs::read_to_string(artifact.with_extension("exe.sig")).unwrap();
    let bytes = std::fs::read(&artifact).unwrap();
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let url = format!(
        "http://{}/fixture-setup.exe",
        listener.local_addr().unwrap()
    );
    let server_bytes = bytes.clone();
    let server = std::thread::spawn(move || {
        for truncated in [false, true] {
            let (mut stream, _) = listener.accept().unwrap();
            stream
                .set_read_timeout(Some(Duration::from_secs(5)))
                .unwrap();
            let mut buffer = [0; 4096];
            let _ = stream.read(&mut buffer).unwrap();
            write!(
                stream,
                "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
                server_bytes.len() + usize::from(truncated)
            )
            .unwrap();
            stream.write_all(&server_bytes).unwrap();
        }
    });
    let (_app, updater) = updater(public.trim());
    let manifest = serde_json::json!({"version":"1.2.0","platforms":{"windows-x86_64":{"url":url,"signature":signature.trim()}}});
    let mut update = updater.restore_update(manifest.clone()).unwrap().unwrap();
    update.timeout = Some(Duration::from_secs(5));
    assert!(update.verify(&bytes).is_ok());
    assert!(update.verify(b"tampered payload").is_err());
    let mut different_version = manifest;
    different_version["version"] = "9.9.9".into();
    let replay = updater.restore_update(different_version).unwrap().unwrap();
    assert!(
        replay.verify(&bytes).is_err(),
        "An old signed artifact must not be advertised as a new release"
    );
    assert_eq!(update.download(|_, _| {}, || {}).await.unwrap(), bytes);
    assert!(update.download(|_, _| {}, || {}).await.is_err());
    server.join().unwrap();
}

#[test]
#[ignore = "Build isolated signed NSIS fixtures with npm run test:updater:build first"]
fn isolated_nsis_fixtures_have_valid_version_bound_signatures() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .join(".cache/update-fixtures");
    let fixtures: serde_json::Value =
        serde_json::from_slice(&std::fs::read(root.join("fixtures.json")).unwrap()).unwrap();
    assert_eq!(fixtures["identifier"], "org.spark.browser.updater-tests");
    let (_app, updater) = updater(fixtures["publicKey"].as_str().unwrap());
    for version in fixtures["versions"].as_array().unwrap() {
        let version = version.as_str().unwrap();
        assert!(semver::Version::parse(version).is_ok());
        let folder = root.join(version);
        let manifest =
            serde_json::from_slice(&std::fs::read(folder.join("updater.json")).unwrap()).unwrap();
        let update = updater.restore_update(manifest).unwrap().unwrap();
        let bytes =
            std::fs::read(folder.join(format!("SparkBrowser_{version}_x64-setup.exe"))).unwrap();
        assert_eq!(
            &bytes[..2],
            b"MZ",
            "Fixture must be an actual Windows installer"
        );
        update.verify(&bytes).unwrap();
    }
}
