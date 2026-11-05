# Extensions (Chrome WebExtensions Support)

Veloce Browser supports loading and running standard Chrome WebExtensions (`manifest.json` v2/v3).

---

## Capabilities

* **Unpacked Extensions:** Load unpacked extension folders directly from your disk or extract `.zip`/`.crx` archives.
* **Persistent Installation:** Installed extensions are preserved in your browser's application data folder and registered in the local SQLite database across restarts.
* **Toolbar Actions:** Extensions declaring `action` or `browser_action` display their icon in the quick actions toolbar and open their popup interfaces upon clicking.
* **Enable / Disable / Remove:** Full lifecycle control to toggle extensions on or off without losing your data, or uninstalling them completely.
* **Cross-Platform Readiness:** Native integration with WebView2 on Windows and an extensible decoupled architecture for Linux (WebKitGTK).

---

## How to Install an Extension

1. Click on the **Extensions** (puzzle piece) icon on the top toolbar or open **Settings -> Extensions**.
2. Click **Load Unpacked**.
3. Select the folder containing the extension's `manifest.json`.
4. The extension will appear in your installed list and its action icon will be pinned to the toolbar.

---

## Cross-Platform Technical Architecture

* **Windows:** Uses Microsoft Edge WebView2's native `ICoreWebView2Profile7` interface (`AddBrowserExtension`), which executes background workers, handles content script injection, and supports Chromium WebExtension APIs.
* **Linux:** Uses an abstracted driver model with full documentation for WebKitGTK in [linux-extensions-guide.md](../architecture/linux-extensions-guide.md).

