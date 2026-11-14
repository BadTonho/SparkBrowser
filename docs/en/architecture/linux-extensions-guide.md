# Linux Extensions Architecture & Development Guide

This guide provides precise technical specifications, architectural blueprints, and actionable instructions for implementing and testing Chrome WebExtensions on Linux within **Spark Browser**.

---

## 1. Architectural Overview: Windows vs Linux

Spark Browser uses native platform webview engines to maintain exceptional speed and resource efficiency without bundling an entire Chromium distribution:

| Dimension | Windows (`#[cfg(windows)]`) | Linux (`#[cfg(target_os = "linux")]`) |
| :--- | :--- | :--- |
| **Engine** | Microsoft Edge WebView2 (Chromium-based) | WebKitGTK (WebKit-based, similar to Safari/Epiphany) |
| **Extension API** | Native `ICoreWebView2Profile7::AddBrowserExtension` | WebKitGTK process-separated extension model |
| **Background Execution** | Handled natively by Chromium runtime | Injected background worker / WebKit web process |
| **Content Script Injection** | Automatic via Chromium profile | `WebKitUserContentManager` / `WebKitWebExtension` |

The Spark backend is decoupled through the `crate::extensions::platform` module abstraction, ensuring full cross-platform compilation and zero compile errors on both operating systems.

---

## 2. WebKitGTK Architecture Details

WebKitGTK isolates web execution into separate operating system processes:
1. **UI Process (Spark Shell & Rust Backend):** Manages windows, tabs, SQLite database, titlebar, and user input.
2. **Web Process (`WebKitWebProcess`):** Renders HTML, executes page JavaScript, and manages DOM trees.

Because of this boundary, extensions running on Linux operate through two complementary strategies:

```
+----------------------------------------------------------------------------------+
|                            Linux UI Process (Spark)                             |
|                                                                                  |
|  - Extension Storage (app_data_dir/extensions/<id>)                             |
|  - SQLite Database (installed_extensions)                                        |
|  - Titlebar Action Icon & Popup Container                                       |
|  - WebKitUserContentManager (manages user scripts & style sheets)                |
+----------------------------------------+-----------------------------------------+
                                         | IPC (D-Bus / WebKit Bridge)
+----------------------------------------v-----------------------------------------+
|                       WebKitWebProcess (Renderer / Tab)                          |
|                                                                                  |
|  - Window DOM (Page context)                                                     |
|  - Injected Content Scripts (matching manifest url patterns)                     |
|  - chrome.* Runtime Shim (chrome.runtime, chrome.storage, chrome.tabs)           |
+----------------------------------------------------------------------------------+
```

---

## 3. Recommended Implementation Roadmap on Linux

When developing on Linux, follow this incremental roadmap:

### Phase 1: Content Scripts & UserStyles Injection (Immediate High Value)
1. Read the parsed `manifest.json` for `content_scripts`:
   * Extract `matches` (e.g. `*://*.youtube.com/*`).
   * Extract `js` and `css` files.
   * Extract `run_at` (`document_start`, `document_end`, `document_idle`).
2. When creating a WebKit webview for a tab in `src-tauri/src/window/webview_manager.rs`, access its `WebKitUserContentManager`:
   * Use `webkit_user_script_new` to register scripts with `WEBKIT_USER_CONTENT_INJECT_ALL_FRAMES`.
   * Use `webkit_user_style_sheet_new` for CSS injection.
3. This immediately supports visual extensions, theme modifiers (like Dark Reader), and ad-blocking script injections.

### Phase 2: Action Popups & UI Actions
1. If the extension declares an `action` or `browser_action` with `default_popup`:
   * The Spark shell renders the action icon in `TitlebarActions.svelte`.
   * Clicking the button opens an internal popover (`ExtensionActionPopover.svelte`) loading `file:///<extension_path>/<popup.html>` or a Tauri internal webview.

### Phase 3: WebKitWebExtension Shared Library (`.so`)
For extensions that require background scripts and full `chrome.runtime` message passing:
1. Create a dedicated dynamic library crate `spark-web-extension` compiling to `libspark_extension.so`.
2. Implement `webkit_web_extension_initialize_with_user_data`.
3. Point WebKit's context to the directory using `webkit_web_context_set_web_extensions_directory`.
4. Expose a JavaScript bridge injecting `window.chrome = { runtime: { ... }, storage: { ... } }`.

---

## 4. Linux Development Setup & Prerequisites

When switching to your Linux environment (Ubuntu / Debian / Fedora / Arch):

### Ubuntu / Debian:
```bash
sudo apt update
sudo apt install -y \
  libwebkit2gtk-4.1-dev \
  libgtk-3-dev \
  libsoup-3.0-dev \
  libjavascriptcoregtk-4.1-dev \
  build-essential \
  curl \
  pkg-config \
  libssl-dev
```

### Fedora:
```bash
sudo dnf install -y \
  webkit2gtk4.1-devel \
  gtk3-devel \
  libsoup3-devel \
  openssl-devel
```

### Arch Linux:
```bash
sudo pacman -S --needed \
  webkit2gtk-4.1 \
  gtk3 \
  libsoup3 \
  base-devel
```

---

## 5. Key Source Code Locations

* **Platform implementation for Linux:**
  [linux.rs](file:///d:/code/ProjetosCode/projetos/navegador/src-tauri/src/extensions/linux.rs)
* **Shared Extension Service:**
  [mod.rs](file:///d:/code/ProjetosCode/projetos/navegador/src-tauri/src/extensions/mod.rs)
* **Manifest Parser (cross-platform):**
  [manifest.rs](file:///d:/code/ProjetosCode/projetos/navegador/src-tauri/src/extensions/manifest.rs)
* **Database Repository:**
  [extensions_repo.rs](file:///d:/code/ProjetosCode/projetos/navegador/src-tauri/src/database/extensions_repo.rs)
* **Frontend Manager & Popover:**
  [ExtensionsModal.svelte](file:///d:/code/ProjetosCode/projetos/navegador/src/lib/components/extensions/ExtensionsModal.svelte)

---

## 6. Verification Commands on Linux

Once in your Linux environment, run:
```bash
# 1. Verify Rust compilation
cargo check --manifest-path src-tauri/Cargo.toml

# 2. Run unit tests
cargo test --manifest-path src-tauri/Cargo.toml

# 3. Run frontend checks
npm run check

# 4. Launch development browser
npm run tauri dev
```

