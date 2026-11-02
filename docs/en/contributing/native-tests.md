# Local native integration

Validation runs locally on demand using the commands in [validation](validation.md). The repository has no GitHub Actions workflow. The Windows native suite exercises the release application through [Tauri WebDriver](https://v2.tauri.app/develop/tests/webdriver/).

## Isolated Windows run

Install Node 24, stable Rust and the normal Tauri prerequisites, then run from the repository root:

```powershell
npm ci
cargo install tauri-driver --version 2.1.0 --locked --root .cache/native-tools
./tools/testing/install-edge-driver.ps1
npm run test:native:build
$env:VELOCE_NATIVE_DRIVER = '.cache/native-tools/edge/msedgedriver.exe'
npm run test:native
```

The Edge driver installer matches the installed WebView2 runtime. Builds use `.cache/native-target` and identifier `org.veloce.browser.native-tests`. Each scenario receives a fresh directory under `.cache/native-runs/` with separate `data`, `profile`, `config`, `cache`, `logs` and `artifacts` folders. Restarts within a scenario reuse those paths. Only the test identifier accepts `VELOCE_NATIVE_TEST_RUN_DIR`, restricted to this root; production ignores it. The runner verifies both runtime identity and data directory before changing state.

Run `npm run test:native -- --list` to list scenarios or `npm run test:native -- --case notes` to run one. With no arguments the suite runs every scenario independently, continuing after failures. Stable `data-testid` selectors do not depend on UI language. Driver ports default to 4444/4445; use `VELOCE_DRIVER_PORT` for another unused pair.

Cleanup runs in `finally`, terminates only the scenario's driver process tree and removes data/profile/config/cache/logs even on failure. Reports and failure screenshots remain in each `artifacts` directory. Cleanup errors are reported without replacing the original failure. Set `$env:VELOCE_NATIVE_FAIL_CASE = 'notes'` for a deliberate post-scenario failure to verify cleanup; remove it afterward. `VELOCE_NATIVE_SCREENSHOTS=1` also retains an updates screenshot in the dialogs scenario.

The suite uses local HTTP fixtures and checks resident tab/form/scroll preservation, SPA history, shield changes, address drafts, dialogs, resize and session restoration. It also exercises named session restoration, keyword encoding, page notes, protected bulk closing, split view replacement/closure/restoration and native permission prompts. Windows test processes receive `--use-fake-device-for-media-stream`; microphone checks use a simulated device. Permission scenarios cover one-time/remembered decisions, reset, navigation cancellation, embedded origins and persistence across restart. Tests use existing commands and real DOM handlers without test-only IPC grants.

Workspace picker regression checks record native page visibility transitions: opening, closing, rapid reopening and reconciliation under a low resident limit must preserve the page behind the picker, including both split panes. Form values, scroll and document load counts must remain unchanged. Shell dialogs must still hide and restore both panes.

The direct runner also accepts a Linux `tauri-driver`/WebKitWebDriver installation through `VELOCE_TAURI_DRIVER` and `VELOCE_NATIVE_DRIVER`; that path has not been validated locally. macOS needs a separate embedded WebDriver service. Multi-monitor DPI transitions, pointer drag gestures, rendering latency and aggregate process memory still require dedicated validation.

The `recovery` scenario force-stops its own test process, verifies all tabs and split focus, then checks a clean restart. `recovery-failure` injects a SQLite trigger only in the stopped, identity-verified scenario database, proves initialization failure keeps the previous snapshot, removes the trigger and retries. The `closed-tabs` and `backup` scenarios cover restoration and reviewed import with persistent preference retries.

The suite also includes `downloads` (durable checkpoints, HTTP ranges, integrity and native controls), `shutdown` (note flushing, changed-risk review, cancellation and SQLite failure), `tab-diagnostics` (real residency and revalidated protection), `zoom` (resident page scale, origins, navigation, shortcuts, restart and backup preferences), and `command-search` (bounded Unicode results, stable selection, original note URLs and reviewed session details). Download fixtures write exclusively under each scenario's `data/downloads`, never the personal Downloads folder. The complete suite has twenty-one independent scenarios. On Windows, `keyboard` sends real input only to the foreground isolated executable and verifies closing/reopening consecutive tabs while modifiers remain pressed; keep the test desktop available. It does not execute installers or use a VM.

## Measurements

After building a production release, run `node tools/testing/release-metrics.mjs` to report the Git revision, toolchain versions, frontend raw/gzip bytes and native executable bytes. These are artifact measurements, not RAM or startup benchmarks. Follow [performance validation](performance.md) for comparisons. `npm run security:check` verifies tracked/staged paths, common secret signatures, ignore rules and whitespace without printing secret contents.

Run `--case media-mute` for native audio activity, background mute and restart defaults.

`--case media-volume` measures artificial audio gain and verifies volume/navigation fallback.

`--case media-pip` validates real native Picture-in-Picture and protection of its source WebView.
