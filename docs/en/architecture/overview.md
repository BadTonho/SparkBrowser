# 🏗️ Architecture & Technology Stack

> 🌍 **Language:** **English** | [Português (pt-BR)](../../pt-BR/architecture/overview.md)

## 1. Core / Backend (100% Rust)
- **Base Framework:** **Tauri v2** + **Wry (Multi-WebView)** + **Tao**
  - Leverages the operating system's native web engine (WebView2 / Chromium on Windows, WebKit on macOS/Linux), producing a tiny **3 MB to 5 MB** executable with near-instant startup.
- **Native Window Effects:** `window-vibrancy`
  - Enables hardware-accelerated translucent blur (**Mica / Acrylic** on Windows 10/11) with zero CPU overhead.
- **Native Ad & Tracker Blocker:** `adblock` (`adblock-rust`, the open-source engine used by Brave)
  - Intercepts and blocks ads and trackers at the network layer directly in Rust.
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
   - Active websites run inside isolated child WebView instances framed inside the browser shell.
   - Real-time synchronization: when the sidebar expands or collapses, Rust dynamically resizes the web content viewport in sync with the animation.
