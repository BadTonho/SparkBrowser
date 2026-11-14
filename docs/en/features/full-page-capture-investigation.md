# Full-page PNG capture: feasibility investigation

**Status:** The Windows probe compiles, but runtime validation is blocked before the test application launches. Linux runtime validation is pending.

## Findings so far

- The current product has no full-page capture feature or capture command. The content WebViews are accessible through Tauri's native WebView handle.
- WebView2 `CapturePreview` captures what the WebView is displaying. WebView2 also exposes the Chrome DevTools Protocol through `CallDevToolsProtocolMethod`; the protocol's `Page.captureScreenshot` accepts `captureBeyondViewport`, but that parameter is marked experimental and the protocol does not guarantee compatibility across versions. See [WebView2 DevTools Protocol](https://learn.microsoft.com/en-us/microsoft-edge/webview2/how-to/chromium-devtools-protocol), [WebView2 API](https://learn.microsoft.com/en-us/microsoft-edge/webview2/reference/win32/icorewebview2?view=webview2-1.0.664.37), and [CDP Page.captureScreenshot](https://chromedevtools.github.io/devtools-protocol/tot/Page/).
- WebKitGTK exposes `get_snapshot` with a `FULL_DOCUMENT` region. The repository locks the Rust binding to `webkit2gtk` 2.0.2, which exposes this region and method. See the [WebKitGTK snapshot API](https://webkitgtk.org/reference/webkit2gtk/stable/method.WebView.get_snapshot.html).
- These API surfaces make full-document capture plausible on both engines, but do not establish behavior for long documents, fixed/sticky elements, lazy resources, frames, or large output images.

## Reproducible probe

The isolated native `capture-probe` scenario loads a loopback fixture with a long document, fixed header/footer, sticky content, a lazy image, a long iframe, changing text and a bottom marker. The guarded probe command captures the active content WebView, writes `artifacts/full-page-probe.png`, and reports image dimensions, byte count, engine and elapsed time. The scenario checks that the image extends beyond the viewport and that the page's scroll position and document height remain unchanged.

On Windows, the probe calls `Page.captureScreenshot` with `captureBeyondViewport: true`. On Linux, it calls WebKitGTK's `FullDocument` snapshot API. The command rejects any application identifier except `org.spark.browser.native-tests` and writes only beneath that scenario's `.cache/native-runs` directory. `cargo check --manifest-path src-tauri/Cargo.toml --lib` and the isolated Windows native build pass. The runtime scenario did not produce a WebDriver session: EdgeDriver reported ready, but the session creation request stalled before the isolated app process appeared. No PNG or visual coverage result was collected, so Windows feasibility remains unconfirmed.

To retry Windows runtime validation, point the harness at the EdgeDriver matching the installed Edge version and run the isolated scenario:

```powershell
$env:SPARK_NATIVE_DRIVER = '.cache/native-tools/edge/msedgedriver.exe'
npm.cmd run test:native -- --case capture-probe
```

## Linux validation

The current development environment only has the Windows Rust target, so Linux runtime behavior is not confirmed here. On a Linux machine with the project's Tauri/WebKitGTK prerequisites installed, record the linked runtime and run:

```sh
pkg-config --modversion webkit2gtk-4.1
npm run test:native:build:fast
npm run test:native -- --case capture-probe
```

The scenario output includes the measured dimensions, PNG size and elapsed time. Inspect `artifacts/full-page-probe.png` in the printed `.cache/native-runs/...` directory for fixed/sticky duplication, lazy-image visibility, iframe coverage, and whether the bottom marker appears. Report the WebKitGTK version and scenario output before closing the cross-platform gate.

## Decision gate

After Windows and Linux results are available, compare coverage, scroll preservation, output dimensions, memory implications and latency. If the Windows CDP route or Linux snapshot has material gaps, present alternatives before planning the product feature. This investigation does not add a user-facing capture flow.
