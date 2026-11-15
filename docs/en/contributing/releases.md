# Signed Releases & Automated GitHub Actions

Releases are published automatically via **GitHub Actions** upon pushing version tags (e.g. `v1.0.0` or `v1.1.0-beta.1`), or built locally. Public releases live in `BadTonho/SparkBrowser`. Both Windows (NSIS installer) and broad Linux distributions (.AppImage, .deb, .rpm, and portable .tar.gz) packages are supported. Both channels retain identifier `org.spark.browser` and share the existing profile/database.

## 1. Automated Releases (GitHub Actions)

When you push a tag matching `v*`, the `.github/workflows/release.yml` workflow triggers automatically:
1. **Windows x64 Build:** Compiles the app, signs the installer using `TAURI_SIGNING_PRIVATE_KEY` stored in repository secrets, and generates `SparkBrowser_<version>_x64-setup.exe`, its `.sig` signature, and the `updater.json` manifest.
2. **Linux x64 Build:** Compiles the app with WebKitGTK and generates multi-distro packages:
   - **`.AppImage` (Universal):** Runs on virtually all Linux distributions (Fedora, Arch Linux, Manjaro, openSUSE, Pop!_OS, Linux Mint, Debian, Ubuntu, Void, etc.) without requiring root or installation.
   - **`.deb` (Debian / Ubuntu):** Native package for Debian, Ubuntu, Pop!_OS, Linux Mint, Elementary OS, and Zorin OS.
   - **`.rpm` (Red Hat / SUSE):** Native package for Fedora, openSUSE, RHEL, CentOS Stream, AlmaLinux, and Rocky Linux.
   - **`portable.tar.gz` (Standalone Archive):** Standalone portable binary tarball ideal for Arch Linux (AUR packaging) and custom environments.
3. **Publication (Draft Mode):** Creates the GitHub Release as a **Draft**, attaches all compiled installers, signatures, and manifests, and automatically marks `-beta.*` tags as **Pre-release**. Being created in draft mode allows you to review the generated release notes and inspect all attached packages privately before publishing them to the public.

### Linux Installation Commands

- **Universal AppImage:**
  ```bash
  chmod +x spark-browser_*.AppImage
  ./spark-browser_*.AppImage
  ```
- **Debian / Ubuntu / Pop!_OS / Mint (`.deb`):**
  ```bash
  sudo apt install ./spark-browser_*_amd64.deb
  ```
- **Fedora / RHEL (`.rpm`):**
  ```bash
  sudo dnf install ./spark-browser-*.rpm
  ```
- **openSUSE (`.rpm`):**
  ```bash
  sudo zypper install ./spark-browser-*.rpm
  ```

### Before creating the tag

The release workflow builds and publishes; it does not run the native regression suite (that suite is a separate, manual workflow, `native.yml`, and has not run successfully on GitHub runners). Validate on your own machine first:

1. Run `npm run validate` and the critical native scenarios with `npm run test:native:critical` (see [native tests](native-tests.md)).
2. Check that the repository is clean and follows the [Zero-Leak policy](rules-and-standards.md#rule-1-zero-sensitive-data-in-version-control).
3. From version 0.1.1 on, make sure `changelog/<version>/` exists with `en.md` and `pt-BR.md` (see the [changelog guide](changelog.md)). Version 0.1.0 has no changelog.

### Step-by-step to publish an update:

```bash
# 1. Update version numbers across package.json, Cargo.toml, and tauri.conf.json
npm run release:version -- v0.2.0

# 2. Review and commit changes
git add .
git commit -m "chore: release v0.2.0"

# 3. Create tag and push to GitHub
git tag v0.2.0
git push origin main --tags
```

GitHub Actions will build and sign the packages and create the release as a **draft**. Nothing is public until you publish it:

1. Open the draft in **Releases** and check that it has the Windows installer (`.exe`, `.sig`, `updater.json`) and every Linux package (`.AppImage`, `.deb`, `.rpm`, `.tar.gz`).
2. Review the generated release notes and edit them if needed.
3. Download and try at least the Windows installer and one Linux package.
4. Click **Publish release**. Beta tags (`-beta.N`) are already marked **Pre-release**.

## 2. Signing identity

Updater signatures use Minisign Ed25519 keys via Tauri's official updater plugin.
- The **Public Key** is embedded into `src-tauri/tauri.conf.json` under `plugins.updater.pubkey`.
- The **Private Key** must be stored strictly in GitHub Secrets as `TAURI_SIGNING_PRIVATE_KEY` (and `TAURI_SIGNING_PRIVATE_KEY_PASSWORD` only if the key has a password; keys created with `npm run release:setup-signing` have none). Never commit or expose private keys in git!

## 3. Local manual builds (Fallback)

1. Run `npm run release:version -- v1.1.0-beta.1` or `npm run release:version -- v1.1.0`. This synchronizes npm metadata, Tauri configuration, Cargo package version and lockfile; it does not create a tag.
2. Run [validation](validation.md), review status/diffs/ignore rules for sensitive content and commit. Create the matching local tag on that commit, for example `git tag v1.1.0-beta.1`.
3. Run `npm run release:build -- v1.1.0-beta.1`. Optional notes: `npm run release:build -- v1.1.0-beta.1 --notes path/to/notes.md`. It requires a clean checkout, matching versions and a tag pointing to HEAD. It runs `validate --native` and checks the checkout again before accessing signing material or packaging; failed checks stop the release. Set up the native drivers and cache Rust dependencies first. Missing Tauri NSIS tools are downloaded on the first build.
   The private signing key is read from `~/.spark-signing/updater.key`, or from the old `~/.veloce-signing/updater.key` when that is where it still lives; `SPARK_SIGNING_KEY_PATH` overrides both (see [Upgrading from the Veloce name](../features/rename-migration.md)).
4. Output is `.cache/release-artifacts/<tag>/`. Upload only `SparkBrowser_<version>_x64-setup.exe`, its `.sig` and `updater.json` to that exact tag's release. Do not upload `build-config.json`, caches, profiles or private signing files. The manifest also embeds the signature.
5. Push the reviewed commit/tag and publish the GitHub release yourself. Mark `*-beta.N` as **Pre-release**; stable releases must not have that flag. Upload all three assets before publishing. A tag alone is not an update.

Artifacts have fixed names without spaces; manifests reference that tag rather than `/latest`. Missing manifests/installers produce distribution errors; manifests explicitly targeting another architecture are skipped. Lookup scans up to 2,000 releases and fails explicitly if exceeded. GitHub rate-limit retry times are respected. The first updater-enabled version requires manual installation; a newer complete signed release is needed for subsequent updates. These tools never push or publish.

## Isolated validation

`cargo test --manifest-path src-tauri/Cargo.toml` includes an integration test that signs artificial bytes with temporary keys and exercises the official plugin against valid signatures, tampering, version replay and interrupted downloads. It never executes an installer.

```powershell
npm run test:updater:build
cargo test --manifest-path src-tauri/Cargo.toml --test updater_signatures isolated_nsis_fixtures -- --ignored
npm run test:native:build
```

The builder creates beta/stable NSIS packages in `.cache/update-fixtures/`, identifier `org.spark.browser.updater-tests`, with a temporary signing key deleted afterward. The extra Rust check verifies both real installers using the official plugin. This reuses the isolated native target cache: rebuild the normal native executable before its suite. No installer is executed or production key used. Isolated test apps disable live GitHub update checks.

Installer execution, upgrade/restart and installed-profile restoration still require a disposable Windows VM, unavailable on the implementation machine. Building packages and verifying signatures do not count as an installer-execution pass. In the VM, install the beta fixture, create artificial workspaces/notes/tabs, close it, install the newer stable fixture and verify the version/data. Keep production profiles and keys out of the VM. Live GitHub distribution additionally requires actual published signed releases; implementation does not publish them.
