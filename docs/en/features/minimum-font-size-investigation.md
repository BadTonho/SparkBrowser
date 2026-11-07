# Minimum font size: feasibility investigation

**Status:** The isolated probe builds. The matching EdgeDriver starts, but the native test harness does not complete WebDriver session creation, so no visual measurements are available. Linux validation is prepared for a later run.

## Findings

- WebKitGTK exposes a native `minimum-font-size` setting in pixels. Its documentation warns that a nonzero value can break page layouts. See [`Settings.set_minimum_font_size`](https://webkitgtk.org/reference/webkit2gtk/stable/method.Settings.set_minimum_font_size.html) and the [`minimum-font-size` property](https://webkitgtk.org/reference/webkit2gtk/stable/property.Settings.minimum-font-size.html).
- The documented Win32 `ICoreWebView2Settings` interface includes zoom control but no minimum-font-size setting. This is a finding about the documented interface, not a claim that no undocumented runtime workaround exists. See the [ICoreWebView2Settings reference](https://learn.microsoft.com/en-us/microsoft-edge/webview2/reference/win32/icorewebview2settings?view=webview2-1.0.4022.49).
- Page zoom is available in the app, but it scales the whole page, including text already above a proposed minimum, and can change responsive layout. It is not equivalent to a minimum font size.
- The isolated `minimum-font-probe` fixture includes 8, 12, 16 and 24 px text, relative text, a link, button, input and iframe. It captures 100% and 150% zoom. On Linux it also compares WebKitGTK settings at 0 px and 12 px, then restores the previous value.

## Windows result

This environment has WebView2 Runtime `157.0.4341.42` and initially had cached EdgeDriver `156.0.4314.58`. Microsoft requires matching versions for WebView2 app automation. A matching EdgeDriver `157.0.4341.42` was downloaded into the ignored test cache, and its `/status` endpoint reported ready. See [Automate and test WebView2 apps with Microsoft Edge WebDriver](https://learn.microsoft.com/en-us/microsoft-edge/webview2/how-to/webdriver).

The probe builds with `npm.cmd run test:native:build:fast`. With the matching driver, the test harness still did not complete WebDriver session creation; no app fixture or captures were produced before the attempt was stopped. No visual claim about WebView2 zoom behavior is made yet. The matching-driver run command is:

```powershell
$env:VELOCE_NATIVE_DRIVER = "$PWD/.cache/native-tools/edge/157/msedgedriver.exe"
npm.cmd run test:native -- --case minimum-font-probe
```

## Linux validation

Record the installed WebKitGTK version and run the isolated probe:

```sh
pkg-config --modversion webkit2gtk-4.1
npm run test:native:build:fast
VELOCE_TAURI_DRIVER="$(command -v tauri-driver)" VELOCE_NATIVE_DRIVER="$(command -v WebKitWebDriver)" VELOCE_DRIVER_PORT=4484 npm run test:native -- --case minimum-font-probe
```

Inspect `minimum-font-probe.json` and the screenshots in the run's `artifacts` directory. Compare 0 px and 12 px to see whether the native setting affects authored sizes below the threshold, while checking controls, the iframe and the 100%/150% zoom captures. The probe restores the previous minimum-font setting after measurement.

## Recommendation and decision gate

The documented engine support is asymmetric: WebKitGTK has a native minimum-font setting, while the documented WebView2 settings interface does not. Zoom is an imperfect substitute, and CSS rewriting is excluded from this investigation. Do not implement a cross-platform minimum-font preference until the WebView2 probe is rerun with a matching driver and the Linux screenshots are reviewed. If Windows still has no supported native equivalent, choose whether to accept a platform limitation or a separately scoped alternative before implementation.
