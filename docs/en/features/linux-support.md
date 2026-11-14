# Linux support

Spark runs on Windows (WebView2) and Linux (WebKitGTK through `webkit2gtk` 2.0). Both platforms share the same Svelte interface and Rust state; platform code lives behind `#[cfg(windows)]` and `#[cfg(target_os = "linux")]`.

## Native parity

| Area | Windows | Linux |
|---|---|---|
| Shell hardening (only shell documents in `main`) | WebView2 `NavigationStarting` | Tauri plugin `on_navigation` guard (`security/linux.rs`) |
| New windows and DevTools in internal views | Denied / development only | Denied / development only |
| Address bar and back/forward tracking (including SPA navigation) | WebView2 source and history events | WebKitGTK `notify::uri`, back-forward list and load events |
| Split view pane focus | WebView2 `GotFocus` | GTK `focus-in-event` |
| Tab mute and audible indicator | WebView2 `IsMuted` / `IsDocumentPlayingAudio` | WebKitGTK `is-muted` / `is-playing-audio` |
| Per-site zoom | Page zoom plus native zoom-change tracking | Page zoom applied on navigation; native pinch or Ctrl+wheel changes are not persisted |
| Per-frame media control, tab volume, Picture-in-Picture | Chromium DevTools Protocol execution contexts | Not available (reported as `media.unavailable`) |

The shell URL on release builds is `tauri://localhost`. Custom schemes have an empty path and an opaque origin in `url::Url`, so shell checks compare scheme, host and port instead of `Url::origin`.

## Requirements

Install the WebKitGTK 4.1 and GTK 3 development packages for your distribution (see the [Linux extensions guide](../architecture/linux-extensions-guide.md)). `WebKitWebDriver` is needed for native tests.

## Validation on Linux

```bash
npm install
npm run validate          # architecture, tests, check, fmt, clippy -D warnings, cargo test, build, security
npm run tauri dev         # manual run
```

Native tests also need `tauri-driver` in the ignored `.cache/native-tools` directory:

```bash
cargo install tauri-driver --locked --root .cache/native-tools
npm run test:native:build:fast
npm run test:native:critical
```

Known limit: the `backup-identities` scenario cannot reach account-container tabs because each container uses its own WebKit data directory, which the WebDriver session does not automate.
