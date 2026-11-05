# Signed Releases & Automated GitHub Actions

Releases are published automatically via **GitHub Actions** upon pushing version tags (e.g. `v1.0.0` or `v1.1.0-beta.1`), or built locally. Public releases live in `BadTonho/navegador`. Both Windows (NSIS installer) and broad Linux distributions (.AppImage, .deb, .rpm, and portable .tar.gz) packages are supported. Both channels retain identifier `org.veloce.browser` and share the existing profile/database.

## 1. Automated Releases (GitHub Actions)

When you push a tag matching `v*`, the `.github/workflows/release.yml` workflow triggers automatically:
1. **Windows x64 Build:** Compiles the app, signs the installer using `TAURI_SIGNING_PRIVATE_KEY` stored in repository secrets, and generates `VeloceBrowser_<version>_x64-setup.exe`, its `.sig` signature, and the `updater.json` manifest.
2. **Linux x64 Build:** Compiles the app with WebKitGTK and generates multi-distro packages:
   - **`.AppImage` (Universal):** Runs on virtually all Linux distributions (Fedora, Arch Linux, Manjaro, openSUSE, Pop!_OS, Linux Mint, Debian, Ubuntu, Void, etc.) without requiring root or installation.
   - **`.deb` (Debian / Ubuntu):** Native package for Debian, Ubuntu, Pop!_OS, Linux Mint, Elementary OS, and Zorin OS.
   - **`.rpm` (Red Hat / SUSE):** Native package for Fedora, openSUSE, RHEL, CentOS Stream, AlmaLinux, and Rocky Linux.
   - **`portable.tar.gz` (Standalone Archive):** Standalone portable binary tarball ideal for Arch Linux (AUR packaging) and custom environments.
3. **Publication (Draft Mode):** Creates the GitHub Release as a **Draft**, attaches all compiled installers, signatures, and manifests, and automatically marks `-beta.*` tags as **Pre-release**. Being created in draft mode allows you to review the generated release notes and inspect all attached packages privately before publishing them to the public.

### Linux Installation Commands

- **Universal AppImage:**
  ```bash
  chmod +x veloce-browser_*.AppImage
  ./veloce-browser_*.AppImage
  ```
- **Debian / Ubuntu / Pop!_OS / Mint (`.deb`):**
  ```bash
  sudo apt install ./veloce-browser_*_amd64.deb
  ```
- **Fedora / RHEL (`.rpm`):**
  ```bash
  sudo dnf install ./veloce-browser-*.rpm
  ```
- **openSUSE (`.rpm`):**
  ```bash
  sudo zypper install ./veloce-browser-*.rpm
  ```

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

GitHub Actions will build, sign, and publish the update automatically.

## 2. Signing identity

Updater signatures use Minisign Ed25519 keys via Tauri's official updater plugin.
- The **Public Key** is embedded into `src-tauri/tauri.conf.json` under `plugins.updater.pubkey`.
- The **Private Key** must be stored strictly in GitHub Secrets as `TAURI_SIGNING_PRIVATE_KEY` (and `TAURI_SIGNING_PRIVATE_KEY_PASSWORD` if password-protected). Never commit or expose private keys in git!

## 3. Local manual builds (Fallback)

1. Run `npm run release:version -- v1.1.0-beta.1` or `npm run release:version -- v1.1.0`. This synchronizes npm metadata, Tauri configuration, Cargo package version and lockfile; it does not create a tag.
2. Run [validation](validation.md), review status/diffs/ignore rules for sensitive content and commit. Create the matching local tag on that commit, for example `git tag v1.1.0-beta.1`.
3. Run `npm run release:build -- v1.1.0-beta.1`. Optional notes: `npm run release:build -- v1.1.0-beta.1 --notes path/to/notes.md`. It requires a clean checkout, matching versions and a tag pointing to HEAD. It runs `validate --native` and checks the checkout again before accessing signing material or packaging; failed checks stop the release. Set up the native drivers and cache Rust dependencies first. Missing Tauri NSIS tools are downloaded on the first build.
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
