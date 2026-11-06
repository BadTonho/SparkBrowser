# 🏗️ Architecture & Technology Stack

> 🌍 **Language:** **English** | [Português (pt-BR)](../../pt-BR/architecture/overview.md)

## 1. Core / Backend (100% Rust)
- **Base Framework:** **Tauri v2** + **Wry (Multi-WebView)** + **Tao**
  - Uses the system WebView runtime. Size and performance vary by build and operating system.
- **Native Window Effects:** `window-vibrancy`
  - Enables hardware-accelerated translucent blur (**Mica / Acrylic** on Windows 10/11) with zero CPU overhead.
- **Privacy Shield:** Rust navigation checks and a shared known-host list.
  - A page script filters fetch/XHR/sendBeacon and applies cosmetic CSS. No adblock-rust dependency or complete network interception is currently included.
- **Local Persistence:** `rusqlite` (Embedded SQLite)
  - High-speed local storage for browsing history, bookmarks, workspaces, and session restoration.

---

## 2. Browser UI & Animation Engine (Frontend)
- **UI Framework:** **Svelte (with TypeScript)**
  - Compiles components into tiny, surgical DOM updates without Virtual DOM overhead, keeping memory usage minimal.
- **Animation Engine:** `svelte/motion` (`spring`, `tweened`) + `svelte/transition` + **Modern CSS**
  - Delivers real spring-physics animations at **60/120 FPS**, smooth tab transitions, micro-interactions, and glassmorphism.

---

## 3. Multi-WebView Window Composition

```
+-----------------------------------------------------------------------+
|  Main Window (Rust + Svelte UI + Native Mica/Acrylic Glass Background)|
|  +-----------------+  +--------------------------------------------+  |
|  |                 |  |                                            |  |
|  |  Animated       |  |  Web Content Frame (Child WebView in Rust) |  |
|  |  Sidebar / Tabs |  |  Renders the active website (e.g. YouTube) |  |
|  |  (Svelte)       |  |  Resizes smoothly alongside UI animations  |  |
|  |                 |  |                                            |  |
|  +-----------------+  +--------------------------------------------+  |
+-----------------------------------------------------------------------+
```

1. **Base Layer (Main Window in Rust + Svelte):**
   - Renders the custom titlebar, collapsible **Sidebar** (or Top Bar), navigation controls, URL bar, and bookmarks over a native translucent glass background.
2. **Web Content Frame (Multi-WebView managed by Rust):**
   - Each awake tab owns a child WebView with its own document state and native back/forward stack. The views use the system engine and share the normal browser profile; this does not imply separate cookie profiles or OS processes.
   - Real-time synchronization: when the sidebar expands or collapses, Rust dynamically resizes the web content viewport in sync with the animation.

## 4. Lifecycle and persistence

Ordinary selection shows the existing tab WebView without navigating again. Hibernation or closing destroys only the affected view. Restored background tabs remain lazy until selected. Some shell dialogs temporarily hide content views to avoid native stacking conflicts. Settings use a transparent native overlay that projects the shell's controls and forwards authenticated actions to their existing controllers; the page, including both split panes, stays visible. The transparent native workspace picker also keeps content visible through opening and closing animations without reloading documents.

Only the completed-page callback commits navigation URLs and history, using the originating tab ID even if a different tab has become active. Denied navigation does not replace the committed address. Session replacement is atomic and participates in workspace-deletion transactions; database failures are reported.

Changing the shield rebuilds content WebViews because their initialization scripts are immutable. The active page reloads, background tabs hibernate, and in-page state/back-forward stacks reset. This explicit operation is described beside the UI controls.
