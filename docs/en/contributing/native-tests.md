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

The Edge driver installer downloads the driver matching the installed WebView2 runtime from Microsoft's distribution endpoint. Native builds use `.cache/native-target` and identifier `org.veloce.browser.native-tests`, separating their application data from the regular browser. The runner checks the runtime identifier before changing state. Do not substitute the production executable. The test profile persists across runs, but its tabs contain only artificial loopback fixtures; the profile is never committed. Driver ports default to 4444/4445; set `VELOCE_DRIVER_PORT` to another unused pair when needed.

The suite uses a local HTTP fixture, checks resident tab switching without form/scroll loss, SPA back/forward, protected hibernation, live Windows shield updates, address drafts, dialog focus/Escape, resizing and session restoration. It uses existing commands and real DOM handlers, with no test-only IPC grants. A small diagnostics field identifies the application configuration without exposing browsing data.

The direct runner also accepts a Linux `tauri-driver`/WebKitWebDriver installation through `VELOCE_TAURI_DRIVER` and `VELOCE_NATIVE_DRIVER`; that path has not been validated locally. macOS needs a separate embedded WebDriver service. Multi-monitor DPI transitions, pointer drag gestures, rendering latency and aggregate process memory still require dedicated validation.

## Measurements

After building a production release, run `node tools/testing/release-metrics.mjs` to report the Git revision, toolchain versions, frontend raw/gzip bytes and native executable bytes. These are artifact measurements, not RAM or startup benchmarks. Follow [performance validation](performance.md) for comparisons. `npm run security:check` verifies tracked/staged paths, common secret signatures, ignore rules and whitespace without printing secret contents.
