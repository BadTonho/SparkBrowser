# 🗺️ Development Roadmap

> 🌍 **Language:** **English** | [Português (pt-BR)](../../pt-BR/roadmap/phases.md)

## Phase 1: Base Prototype (MVP) — ✅ Completed
- [x] Initialize **Tauri v2 (Rust) + Svelte + TypeScript** project structure.
- [x] Configure frameless custom window (*custom titlebar*) with translucent blur (`window-vibrancy`).
- [x] Implement basic **Multi-WebView** management in Rust (browser UI shell + active website viewport).
- [x] Add core navigation controls: URL/Search input, Back, Forward, and Reload.

## Phase 2: Multi-Tab System & Modern Animations — ✅ Completed
- [x] Build multi-tab lifecycle management in Rust (create, switch, close, track URL/title).
- [x] Implement spring-physics animations (`svelte/motion`) for tab transitions and collapsible sidebar.
- [x] Create the animated floating **Command Bar (`Ctrl + T` / `Ctrl + K`)**.

## Phase 3: Performance, Privacy & Persistence — ✅ Completed
- [x] Integrate `rusqlite` for local History, Bookmarks, and Session Restoration.
- [x] Integrate native Rust Ad & Tracker Blocker (`privacy/adblock_engine.rs`).
- [x] Implement automatic and manual **Tab Hibernation** for RAM optimization.

## Phase 4: Polish & Advanced Capabilities — ✅ Completed
- [x] Add **Workspaces** (*Personal*, *Work*, *Study*, *Gaming*) and customizable dynamic themes.
- [x] Build a modern animated **Download Manager (`Ctrl + J`)** backed by SQLite.
- [x] Add **Zen Focus Mode (`Alt + Z`)**, customizable keyboard shortcuts, and **Settings Panel (`Ctrl + ,`)**.
