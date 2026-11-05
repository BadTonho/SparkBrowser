# Extensions (Chrome WebExtensions Support)

Veloce Browser supports loading and running standard Chrome WebExtensions (`manifest.json` v2/v3).

---

## Capabilities

* **Chrome Web Store Installer:** Paste any Chrome Web Store URL or 32-character extension ID to download, unpack, and install directly.
* **Archive Installer (.zip / .crx):** Directly install packaged extensions from local `.zip` or `.crx` (CRX2 and CRX3) files.
* **Unpacked Extensions:** Load unpacked extension folders directly from your disk.
* **Persistent Installation:** Installed extensions are preserved in your browser's application data folder and registered in the local SQLite database across restarts.
* **Toolbar Actions:** Extensions declaring `action` or `browser_action` display their icon in the quick actions toolbar and open their popup interfaces upon clicking.
* **Enable / Disable / Remove:** Full lifecycle control to toggle extensions on or off without losing your data, or uninstalling them completely.
* **Cross-Platform Readiness:** Native integration with WebView2 on Windows and an extensible decoupled architecture for Linux (WebKitGTK).

---

## How to Install an Extension

### 1. From the Chrome Web Store
1. Click on the **Extensions** (puzzle piece) icon on the top toolbar.
2. In the input box, paste the Chrome Web Store URL (e.g., `https://chromewebstore.google.com/detail/...`) or the 32-character extension ID.
3. Click **Install**. The extension will be downloaded, validated, extracted, and activated automatically.

### 2. From a .zip or .crx Archive
1. Click on **Load .zip / .crx** in the Extensions modal.
2. Select the archive file from your file manager.
3. Veloce will extract the package into a managed folder and register the extension.

### 3. Load Unpacked
1. Click **Load Unpacked**.
2. Select the folder containing the extension's `manifest.json`.
3. The extension will appear in your installed list.

---

## Cross-Platform Technical Architecture

* **Windows:** Uses Microsoft Edge WebView2's native `ICoreWebView2Profile7` interface (`AddBrowserExtension`), which executes background workers, handles content script injection, and supports Chromium WebExtension APIs.
* **Linux:** Uses an abstracted driver model with full documentation for WebKitGTK in [linux-extensions-guide.md](../architecture/linux-extensions-guide.md).

