# 📂 Project Directory Structure

> 🌍 **Language:** **English** | [Português (pt-BR)](../../pt-BR/architecture/project-structure.md)

The repository strictly follows a modular separation of concerns across both the **Svelte Frontend (`src/`)** and the **Rust Core (`src-tauri/`)**.

---

## 1. Frontend Architecture (`src/`)

```text
src/
├── App.svelte                         # Root layout, auto-hibernation timer & global shortcuts
├── main.ts                            # Application entry point
├── styles/
│   └── theme.css                      # Glassmorphism design tokens & dynamic CSS theme variables
└── lib/
    ├── types/
    │   └── browser.ts                 # Shared interfaces (TabItem, WorkspaceItem, DownloadItem, etc.)
    ├── services/
    │   └── tauri-bridge.ts            # IPC wrapper for navigation, tabs, workspaces, downloads & SQLite
    ├── stores/
    │   ├── browserStore.ts            # Multi-tab reactive state, hibernation actions & sidebar spring motion
    │   ├── commandBarStore.ts         # Floating Command Bar state & spring-physics scale animation
    │   ├── workspaceStore.ts          # Workspaces state & dynamic CSS accent theme injection
    │   ├── downloadsStore.ts          # Download Manager state & drawer controls (Ctrl+J)
    │   ├── settingsStore.ts           # Settings modal (Ctrl+,), Zen Mode (Ctrl+Shift+Z) & hibernation timer
    │   ├── persistenceStore.ts        # SQLite Bookmarks & Browsing History reactive state (Ctrl+H)
    │   └── privacyStore.ts            # Native Rust AdBlock Shield & RAM optimization metrics
    └── components/
        ├── titlebar/
        │   └── Titlebar.svelte        # Navigation/search toolbar, horizontal workspace switcher, browser tools and window controls
        ├── bookmarks/
        │   └── BookmarkBar.svelte     # Horizontally scrollable bookmarks row below the search toolbar
        ├── sidebar/
        │   └── Sidebar.svelte         # Vertical layout sidebar with workspaces, bookmarks and workspace-filtered tabs
        ├── workspaces/
        │   └── WorkspaceSwitcher.svelte # Workspace switcher for the horizontal toolbar and vertical sidebar
        ├── tabs/
        │   ├── TabList.svelte         # Workspace-filtered vertical animated tab list
        │   └── HorizontalTabBar.svelte # Horizontal top tab bar
        ├── privacy/
        │   └── PrivacyShieldCard.svelte # AdBlock Shield toggle, tracker counter & 1-click RAM optimizer
        ├── history/
        │   └── HistoryDrawer.svelte   # Slide-over SQLite Browsing History drawer (Ctrl+H)
        ├── downloads/
        │   └── DownloadsDrawer.svelte # Slide-over Download Manager drawer (Ctrl+J)
        ├── settings/
        │   ├── SettingsModal.svelte   # Browser settings, theme picker, Zen Mode & shortcuts modal (Ctrl+,)
        │   └── settings-sections.ts   # One loader per category: each section is read when first shown
        ├── commandbar/
        │   └── CommandBar.svelte      # Spotlight-style floating Command Bar (Ctrl+T / Ctrl+K)
        └── viewport/
            └── ViewportFrame.svelte   # Framed/Zen viewport syncing bounds with active Rust WebView
```

---

## 2. Rust Core Architecture (`src-tauri/`)

```text
src-tauri/
├── Cargo.toml                         # Rust dependencies (tauri v2, window-vibrancy, rusqlite bundled)
├── tauri.conf.json                    # Frameless & transparent window configuration
├── capabilities/
│   └── default.json                   # Fine-grained Tauri v2 security permissions
└── src/
    ├── main.rs                        # Binary entry point
    ├── lib.rs                         # Tauri builder setup, SQLite init, session restore & commands
    ├── database/
    │   ├── mod.rs
    │   ├── connection.rs              # SQLite DB init inside OS app_data_dir (WAL mode)
    │   ├── history_repo.rs            # Browsing history CRUD operations
    │   ├── bookmarks_repo.rs          # Bookmarks CRUD & default seed operations
    │   ├── downloads_repo.rs          # Download records CRUD operations
    │   └── session_repo.rs            # Automatic workspace-aware session persistence & lazy restore
    ├── privacy/
    │   ├── mod.rs
    │   └── adblock_engine.rs          # Known-host navigation filtering and shared shield script
    ├── state/
    │   ├── mod.rs
    │   ├── browser_state.rs           # Thread-safe workspaces, tabs, SQLite connection & AdBlock stats
    │   ├── tab_model.rs               # TabInfo struct with workspace_id, hibernation & activity timestamps
    │   └── workspace_model.rs         # WorkspaceItem definitions & accent palettes
    ├── window/
    │   ├── mod.rs
    │   ├── vibrancy.rs                # Native OS Mica / Acrylic glass effects
    │   └── webview_manager.rs         # Multi-WebView lifecycle, wake-on-select & shield injection
    └── commands/
        ├── mod.rs
        ├── navigation.rs              # Active tab navigation IPC commands
        ├── tabs.rs                    # Multi-tab, reordering (reorder_tabs), cross-workspace assignment & RAM hibernation IPC commands
        ├── workspaces.rs              # Workspace switching & context filtering IPC commands
        ├── downloads.rs               # Download Manager IPC commands
        ├── persistence.rs             # SQLite History & Bookmarks IPC commands
        └── privacy.rs                 # AdBlock Shield & privacy stats IPC commands
```

## 3. Lifecycle, downloads, and regression tests

- `window/keyboard_bridge.rs`: page/native keyboard integration.
- `window/internal_overlays.rs`: native overlay views and their internal message transport.
- `window/url_input.rs`: URL/search resolution.
- `window/content_lifecycle.rs`: rebuilding views after privacy-policy changes.
- `commands/overlays.rs`: overlay visibility commands, separate from tab operations.
- `downloads/transfer.rs`: HTTP transfers, temporary files, and collision-safe persistence.
- `privacy/content_shield.js`: request hooks generated with Rust's shared host list.
- `lib/stores/workspaceSelectionStore.ts`: shared selection without a module import cycle.
- `lib/components/status/BrowserErrorBanner.svelte`: surfaced backend and asynchronous errors.
- `tests/`: JavaScript regression tests; Rust tests remain beside their modules.
- `tools/vendor/deepmerge/`: licensed, locally patched build dependency.
- `updates/`, `commands/updates.rs`, `database/updates_repo.rs`: native release selection, signed installation and channel preferences.
- `tools/release/`: local version synchronization, signing setup and release packaging; `src-tauri/tests/updater_signatures.rs` exercises the official updater with isolated keys.
