# Forced dark mode for pages: feasibility investigation

> **Outcome:** the feature was built with an injected, light-pages-only filter. See [Dark mode for websites](forced-dark-mode.md).

**Status:** The isolated probe builds on Windows, but its visual runtime check is blocked while WebDriver creates the application session. Linux validation is pending.

## Findings

- The browser's existing dark appearance changes browser chrome and settings; it does not force page content into a dark palette.
- WebView2 `PreferredColorScheme` can select `dark`, `light` or `auto` for a profile. It sets `prefers-color-scheme`; pages must provide dark styles for their authored surfaces to change. See the [WebView2 profile API](https://learn.microsoft.com/en-us/microsoft-edge/webview2/reference/win32/icorewebview2profile).
- WebKit documents the same page opt-in model and says it does not automatically darken arbitrary page content. See [Dark Mode Support in WebKit](https://webkit.org/blog/8840/dark-mode-support-in-webkit/).
- The isolated `dark-mode-probe` covers a page with dark styles, a light page without dark styles, and an already dark page. Each has local SVG imagery, a generated canvas video, dynamic text, and a test-only CSS inversion variant. The probe records computed colors and `matchMedia`, then retains screenshots and JSON under `.cache/native-runs/.../artifacts`.
- The probe command is restricted to the isolated `org.spark.browser.native-tests` application and restores its previous color-scheme value. No product preference, site exception, UI, or public product behavior was added.

## Windows result

`cargo check --manifest-path src-tauri/Cargo.toml --lib` and `npm.cmd run test:native:build:fast` pass. Runtime validation did not reach the fixture. On this Windows 10.0.26300 environment, EdgeDriver 156.0.4314.58 reports ready, but `POST /session` fails with `Microsoft Edge failed to start: crashed (DevToolsActivePort file doesn't exist)` before a test WebView or screenshot is available. Therefore the visual behavior of WebView2 and the experimental CSS transformation remain unmeasured here.

## Linux validation

On Linux with the project's Tauri, GTK and WebKitGTK prerequisites installed, record the WebKitGTK runtime and run:

```sh
pkg-config --modversion webkit2gtk-4.1
npm run test:native:build:fast
SPARK_TAURI_DRIVER="$(command -v tauri-driver)" SPARK_NATIVE_DRIVER="$(command -v WebKitWebDriver)" SPARK_DRIVER_PORT=4484 npm run test:native -- --case dark-mode-probe
```

The Linux probe temporarily switches the isolated test application's GTK dark preference and restores the previous value. The test does not write desktop appearance settings. Inspect the retained screenshots and `dark-mode-probe.json` in the printed run directory. Confirm that the supported page reacts to the dark preference, the light-only page stays light, the authored dark page remains legible, and the transformation's image/video colors and dynamic content are acceptable. Record the WebKitGTK version and the artifact path with the results.

## Decision gate

Do not treat the native color-scheme signal as a complete forced-dark feature: pages that do not supply dark styles will stay as authored. Compare the actual Windows and Linux artifacts before selecting an implementation strategy. If the visual transformation damages media, controls, or already-dark pages, document those limitations and present alternatives before planning product settings or site exceptions.
