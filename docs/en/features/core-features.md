# ✨ Core Features & Visual Experience

> 🌍 **Language:** **English** | [Português (pt-BR)](../../pt-BR/features/core-features.md)

## 1. ⚡ Floating Command Bar (`Ctrl + T` / `Ctrl + K`)
- A Spotlight-inspired, spring-animated floating bar centered on the screen.
- Allows instant URL navigation, web search, switching between open tabs, and executing quick browser actions.

## 2. 💤 Smart Tab Hibernation (RAM Saver)
- Managed natively in Rust.
- Automatically suspends background tabs after a configurable period of inactivity, freeing up system RAM while keeping the tab visible and ready to restore in the sidebar.

## 3. 🛡️ Native Rust AdBlock & Privacy
- Powered by `adblock-rust` to block intrusive ads, pop-ups, and tracking scripts before pages even render.
- Drastically improves page load times and reduces bandwidth and memory consumption.

## 4. 🎨 Workspaces & Dynamic Themes
- Group tabs by context (e.g., *Work*, *Study*, *Personal*, *Gaming*).
- Hold `Alt` for two seconds to open the centered workspace picker. Choose a workspace while holding Alt, or release Alt, press `Esc`, or click outside to dismiss it.
- `Alt + 1` through `Alt + 9` switch immediately to the first nine workspaces in their displayed order and show a brief confirmation.
- The picker and confirmation can appear temporarily in Zen Mode; closing them leaves Zen Mode enabled.
- In Browser Settings, rename a workspace, change its emoji and theme colors, create workspaces, or delete them. Personal is protected.
- When deleting a workspace with open tabs, move them to another workspace or close them; closed tabs remain in browsing history.
- Each workspace has its own accent colors and subtle gradient tints on the translucent window frame. Editing a workspace in Settings does not activate it.

## 5. 📐 Zen / Focus Mode
- A single shortcut hides all browser toolbars and sidebars, giving **100% of the screen** to the active website.
- In Vertical layout, exiting Zen restores the sidebar.

## 6. 🔀 Drag-and-Drop Tab Reordering & Workspace Assignment
- Reorder tabs in the horizontal tab bar or the vertical sidebar using drag-and-drop.
- Quick tab navigation: press `Ctrl + 1` through `Ctrl + 8` to jump to a specific tab, and `Ctrl + 9` to jump to the last tab.
- Global keyboard shortcuts: press `Alt + ↑` and `Alt + ↓` (or `Ctrl + Shift + ↑` / `↓`) anywhere in the application to move the active tab up or down immediately.
- In Vertical layout, drag tabs onto Workspace pills (*Personal*, *Work*, *Study*, *Gaming*) to reassign them between contexts.
- Real-time Svelte state updating coupled with automatic Rust SQLite session persistence (`reorder_tabs`, `move_tab_to_workspace`).

## 7. 📑 Horizontal and Vertical Browser Layouts
- Choose **Horizontal** or **Vertical** in Settings (`Ctrl + ,`); the selection is remembered across app restarts.
- **Horizontal:** Tabs stay at the top, workspace buttons sit beside the search bar, and saved bookmarks appear in a horizontal bar below it.
- **Vertical:** Workspaces, saved bookmarks, and vertical tabs stay together in the collapsible sidebar.

## 8. Organized Browser Settings
- Settings are split into **Appearance & Layout**, **Search & Navigation**, **Workspaces & Theme**, **Performance & Privacy**, and **Keyboard Shortcuts**.
- Use the fixed category rail to switch between panels; only the selected panel is shown and scrolls when its content is taller than the window.
- The last selected category is remembered when the dialog is reopened and across app restarts.
