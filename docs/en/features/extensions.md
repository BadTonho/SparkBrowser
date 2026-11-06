# Extensions (Chrome WebExtensions Support)

On Windows, Veloce Browser can load Chrome WebExtensions through WebView2; compatibility depends on the runtime and extension. Extension installation and management are not yet supported on Linux and return an error.

---

## Capabilities

* **1-Click Chrome Web Store Install:** When browsing any extension on `chromewebstore.google.com`, an interactive **`[🧩 Install in Veloce]`** button automatically appears in the Omnibox. Clicking it downloads, extracts, validates, and activates the extension with a single click.
* **CRX Download Interceptor:** Clicking Google's "Add to Chrome" button is natively intercepted before the WebView2 download barrier, routing the package directly to Veloce's background installer.
* **Chrome Web Store Installer (Manual):** Paste any Chrome Web Store URL or 32-character extension ID into the Extensions modal to download, unpack, and install directly.
* **Archive Installer (.zip / .crx):** Directly install packaged extensions from local `.zip` or `.crx` (CRX2 and CRX3) files.
* **Unpacked Extensions:** Load unpacked extension folders directly from your disk.
* **Persistent Installation:** Installed extensions are preserved in your browser's application data folder and registered in the local SQLite database across restarts.
* **Toolbar Actions:** Extensions declaring `action` or `browser_action` display their icon in the quick actions toolbar and open their popup interfaces upon clicking.
* **Enable / Disable / Remove:** Full lifecycle control to toggle extensions on or off without losing your data, or uninstalling them completely.
* **Platform availability:** Native integration uses WebView2 on Windows. Linux support remains planned.

---

## How to Install an Extension

### 1. 1-Click from the Chrome Web Store (Recommended)
1. Navigate to any extension page on the Chrome Web Store (e.g., `https://chromewebstore.google.com/detail/...`).
2. An **`[🧩 Install in Veloce]`** button will appear directly inside the address bar (Omnibox).
3. Click it once: Veloce downloads and activates the extension immediately, and a floating toast confirms the installation.

### 2. From the Extensions Modal
1. Click on the **Extensions** (puzzle piece) icon on the top toolbar.
2. In the input box, paste the Chrome Web Store URL or the 32-character extension ID.
3. Click **Install**. The extension will be downloaded, validated, extracted, and activated automatically.

### 3. From a .zip or .crx Archive
1. Click on **Load .zip / .crx** in the Extensions modal.
2. Select the archive file from your file manager.
3. Veloce will extract the package into a managed folder and register the extension.

### 4. Load Unpacked
1. Click **Load Unpacked**.
2. Select the folder containing the extension's `manifest.json`.
3. The extension will appear in your installed list.

---

## Cross-Platform Technical Architecture

* **Windows:** Uses Microsoft Edge WebView2's native `ICoreWebView2Profile7` interface (`AddBrowserExtension`), which executes background workers, handles content script injection, and supports Chromium WebExtension APIs.
* **Linux:** Extension installation and management are unavailable. The proposed WebKitGTK integration is described in [linux-extensions-guide.md](../architecture/linux-extensions-guide.md).


Archive imports are limited to 64 MiB compressed, 32 MiB per entry, 128 MiB extracted in total, and 2,048 entries.
