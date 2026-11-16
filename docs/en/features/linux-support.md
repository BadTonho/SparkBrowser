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
| Per-site zoom | Page zoom plus native zoom-change tracking | Saved zoom applied when a page commits; Ctrl+wheel steps the zoom and saves it (`zoom/linux.rs`) |
| Site icons (favicons) | WebView2 `FaviconChanged` / `GetFavicon` | WebKitGTK `notify::favicon` with the favicon database enabled per profile; stored as a small PNG (`window/favicon/linux.rs`) |
| Network blocking (shield) | WebView2 resource-request handler | Filter lists compiled into a WebKit content rule list per page (`privacy/content_filter_linux.rs`) |
| Site permissions (camera, microphone, location, notifications) | Remembered in the WebView2 profile | WebKitGTK `permission-request`; choices saved in the profile database (`site_permissions/linux.rs`) |
| Download progress and cancel | WebView2 download operations | WebKitGTK download object tracked from the engine download event (`downloads/native_linux.rs`) |
| Gmail widget credentials and cookies | Windows Credential Manager, WebView2 cookie manager | Secret Service through `secret-tool`, WebKitGTK cookie manager |
| Tab memory | WebView2 per-process frame list | Resident memory of the `WebKitWebProcess` children, split between resident tabs |
| Tab volume (0-200%) | Chromium DevTools Protocol plus stream capture for amplification | Page script through a WebKit script message handler; attenuation by element volume, amplification through WebAudio |
| Floating Picture-in-Picture player | Stream captured from the video, source image hidden | Frames copied into a canvas in the popup; the source stays visible |

### Media on WebKitGTK

WebKitGTK has no `HTMLMediaElement.captureStream()` and, in common builds, no WebRTC, so a `MediaStream` cannot be displayed in a `<video>`. The Linux paths differ from Windows in these ways:

- **Volume above 100%** routes the element through a WebAudio gain node. A media element source can never be released, so the graph stays for the life of the document (gain returns to 1 at 100%). It is only used for media WebAudio may read (same origin, `blob:`, or CORS); otherwise the setting reports `limited` and the page's audio is left untouched.
- **Picture-in-Picture** works with any non-protected video, including cross-origin ones, because the popup draws frames with `drawImage`. The original video stays visible because WebKit pauses invisible muted-autoplay videos. The frameless window is created as a related webview and WebKit rebuilds its document after `window.open()`, so the player is built after that happens.
- **Window behavior:** always-on-top, taskbar hiding and the shadow option are best effort and depend on the window manager (Wayland ignores always-on-top for ordinary windows).
- **Engine echo:** WebKitGTK reports the audio stream's remembered volume as `volumechange`. The controller ignores such changes unless the user interacted with the page, so the tab volume never becomes the page's own volume.

The shell URL on release builds is `tauri://localhost`. Custom schemes have an empty path and an opaque origin in `url::Url`, so shell checks compare scheme, host and port instead of `Url::origin`.

### Differences that remain on Linux

- **Shield counters:** WebKit applies content rules without reporting each blocked request, so the blocked-requests counter only counts blocked navigations. The rules are attached to a page when it starts loading and only for pages the shield protects; a site exception takes effect on the next load. `GET` tracking-parameter cleaning and `Sec-GPC` headers need request rewriting and are not available.
- **Downloads:** WebKitGTK cannot pause or resume a transfer, so those actions stay hidden; cancel and progress work.
- **Permissions:** screen capture and other request types are denied. WebKit does not report the frame that asked, so the dialog names the page origin.
- **Automatic updates:** Windows only. Linux installs come from the package manager (deb, rpm) or a new AppImage.
- **Location widget:** unavailable; WebKitGTK has no system location API without GeoClue.
- **Window appearance:** Mica/Acrylic have no Linux equivalent, and GTK logs `gtk_window_set_titlebar() called on a realized window`, which is harmless.

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
npm run test:native -- --case media-linux   # volume, mute and Picture-in-Picture on WebKitGTK
```

Known limit: the `backup-identities` scenario cannot reach account-container tabs because each container uses its own WebKit data directory, which the WebDriver session does not automate.
