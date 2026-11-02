# Local signed Windows releases

Releases are built locally without GitHub Actions, a hosted update server or automatic pushes. Public releases live in `BadTonho/navegador`. The initial format is NSIS for Windows x64, installed for the current user. Both channels retain identifier `org.veloce.browser` and share the existing profile/database.

## Signing identity

Run `npm run release:setup-signing` once for a new signing identity. It creates `%USERPROFILE%\.veloce-signing\updater.key` and its `.pub` companion, then embeds the public verification key. This checkout already has a public key: on another machine, restore its matching private-key backup; setup refuses generating a replacement. Back up that directory securely: losing the private key prevents signing updates trusted by existing installations. Never commit, upload, paste into chat or print the private key/password. The public verification key is safe to distribute.

Set `VELOCE_SIGNING_KEY_PATH` for an existing key outside the checkout, keeping its `.pub` companion beside it. Setup refuses replacing a different configured public key; rotation needs a migration for installed clients. The default generated key has no passphrase and relies on filesystem access protection. For a password-protected key, provide `TAURI_SIGNING_PRIVATE_KEY_PASSWORD` securely in the build environment, never in versioned files or command arguments. The build tool passes the key path without printing its contents.

Updater signatures are separate from Windows Authenticode certificates. This setup does not provide a Windows publisher certificate or suppress reputation prompts. `requireSignedVersion` is enabled: signatures must bind the application version.

## Publishing

1. Run `npm run release:version -- v1.1.0-beta.1` or `npm run release:version -- v1.1.0`. This synchronizes npm metadata, Tauri configuration, Cargo package version and lockfile; it does not create a tag.
2. Run [validation](validation.md), review status/diffs/ignore rules for sensitive content and commit. Create the matching local tag on that commit, for example `git tag v1.1.0-beta.1`.
3. Run `npm run release:build -- v1.1.0-beta.1`. Optional notes: `npm run release:build -- v1.1.0-beta.1 --notes path/to/notes.md`. It requires a clean checkout, matching versions and a tag pointing to HEAD. Missing Tauri NSIS tools are downloaded on the first build.
4. Output is `.cache/release-artifacts/<tag>/`. Upload only `VeloceBrowser_<version>_x64-setup.exe`, its `.sig` and `updater.json` to that exact tag's release. Do not upload `build-config.json`, caches, profiles or private signing files. The manifest also embeds the signature.
5. Push the reviewed commit/tag and publish the GitHub release yourself. Mark `*-beta.N` as **Pre-release**; stable releases must not have that flag. Upload all three assets before publishing. A tag alone is not an update.

Artifacts have fixed names without spaces; manifests reference that tag rather than `/latest`. Missing manifests/installers produce distribution errors; manifests explicitly targeting another architecture are skipped. Lookup scans up to 2,000 releases and fails explicitly if exceeded. GitHub rate-limit retry times are respected. The first updater-enabled version requires manual installation; a newer complete signed release is needed for subsequent updates. These tools never push or publish.

## Isolated validation

`cargo test --manifest-path src-tauri/Cargo.toml` includes an integration test that signs artificial bytes with temporary keys and exercises the official plugin against valid signatures, tampering, version replay and interrupted downloads. It never executes an installer.

```powershell
npm run test:updater:build
cargo test --manifest-path src-tauri/Cargo.toml --test updater_signatures isolated_nsis_fixtures -- --ignored
npm run test:native:build
```

The builder creates beta/stable NSIS packages in `.cache/update-fixtures/`, identifier `org.veloce.browser.updater-tests`, with a temporary signing key deleted afterward. The extra Rust check verifies both real installers using the official plugin. This reuses the isolated native target cache: rebuild the normal native executable before its suite. No installer is executed or production key used. Isolated test apps disable live GitHub update checks.

Installer execution, upgrade/restart and installed-profile restoration still require a disposable Windows VM, unavailable on the implementation machine. Building packages and verifying signatures do not count as an installer-execution pass. In the VM, install the beta fixture, create artificial workspaces/notes/tabs, close it, install the newer stable fixture and verify the version/data. Keep production profiles and keys out of the VM. Live GitHub distribution additionally requires actual published signed releases; implementation does not publish them.
