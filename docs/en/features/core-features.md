# ✨ Core Features & Visual Experience

> 🌍 **Language:** **English** | [Português (pt-BR)](../../pt-BR/features/core-features.md)

## 1. ⚡ Floating Command Bar (`Ctrl + K` by default)
- A Spotlight-inspired, spring-animated floating bar centered on the screen.
- Allows instant URL navigation, web search, switching between open tabs, and executing quick browser actions.

## 2. 💤 Smart Tab Hibernation (RAM Saver)
- Managed natively in Rust.
- Automatically suspends background tabs after 5, 10, or 30 minutes, or with adaptive **Auto** mode. Auto hibernates regular tabs after 10 minutes and tabs activated at least 10 times in the last 7 days after 5 hours of inactivity.
- **Off** disables automatic hibernation while keeping manual hibernation available. Local activation timestamps are kept in SQLite for the 7-day adaptive window without storing page URLs.

## 3. Privacy Shield
- Rust rejects navigation to exact known tracker hosts and their subdomains.
- A best-effort page script filters fetch (including URL/Request inputs), XHR, and sendBeacon, and adds cosmetic CSS. This is not a complete network blocker: parser-loaded scripts/images, service workers, and hostile page modifications can bypass page-level hooks.
- On Windows, changing protection updates live request wrappers and replaces an owned future-document policy script without reloading WebViews, preserving page state and native back/forward stacks. Other engines rebuild content WebViews; the active page reloads and background tabs hibernate. Page state and native back/forward stacks reset on that fallback; changes are refused while detected forms, media or downloads need protection.
- The tracker count currently measures denied navigations only, not all filtered subresources.

## 4. 🎨 Workspaces & Dynamic Themes
- Group tabs by context (e.g., *Work*, *Study*, *Personal*, *Gaming*).
- In **Appearance & Layout**, choose permanent workspace buttons in the horizontal toolbar or a floating picker opened by holding the configured modifier (Alt by default) for 0.4 seconds. In Popup mode, release the modifier, press `Esc`, or click outside to dismiss it.
- `Alt + 1` through `Alt + 9` switch immediately to the first nine workspaces in their displayed order and show a brief confirmation.
- These workspace shortcuts can be changed in Browser Settings; the picker uses a configurable held modifier key.
- The picker and confirmation can appear temporarily in Zen Mode; closing them leaves Zen Mode enabled.
- In Browser Settings, rename a workspace, change its emoji and theme colors, create workspaces, or delete them. Personal is protected.
- When deleting a workspace with open tabs, move them to another workspace or close them; closed tabs remain in browsing history.
- The default browser palette uses graphite, gray, and white. Built-in workspaces start with neutral accents and remain customizable; each workspace keeps its own accent colors and subtle gradient tints on the translucent window frame. Editing a workspace in Settings does not activate it.

## 5. 📐 Zen / Focus Mode
- A single shortcut hides all browser toolbars and sidebars, giving **100% of the screen** to the active website.
- In Vertical layout, exiting Zen restores the sidebar.

## 6. 🔀 Drag-and-Drop Tab Reordering & Workspace Assignment
- Reorder tabs in the horizontal tab bar or the vertical sidebar using drag-and-drop.
- Quick tab navigation: press `Ctrl + 1` through `Ctrl + 8` to jump to a specific tab, and `Ctrl + 9` to jump to the last tab.
- By default, press `Alt + ↑` or `Alt + ↓` (or `Ctrl + Shift + ↑` / `↓`) to move the active tab. Each shortcut can be customized in Browser Settings.
- In Vertical layout, drag tabs onto Workspace pills (*Personal*, *Work*, *Study*, *Gaming*) to reassign them between contexts.
- Real-time Svelte state updating coupled with automatic Rust SQLite session persistence (`reorder_tabs`, `move_tab_to_workspace`).

## 7. 📑 Horizontal and Vertical Browser Layouts
- Choose **Horizontal** or **Vertical** in Settings (`Ctrl + ,`); the selection is remembered across app restarts.
- **Horizontal:** Tabs stay at the top, and saved bookmarks appear in a horizontal bar below the search bar. The bar is hidden when there are no bookmarks or folders. Choose whether workspace buttons sit beside the search bar or appear in a floating picker when holding Alt. Toggle the bookmarks bar from Appearance & Layout or the top toolbar; hiding it does not remove bookmarks.
- **Vertical:** Workspaces, saved bookmarks, and vertical tabs stay together in the collapsible sidebar; the workspace display preference only changes the horizontal layout.
- Save a page with the address-bar star, then choose its title and folder. Selecting an existing bookmark opens the same editor so it can be renamed, moved to another folder, or removed.
- Create, rename, and delete one-level bookmark folders. Folders appear as menus in the horizontal bookmarks bar and expandable sections in the vertical sidebar; deleting a folder moves its bookmarks to the root.
- Right-click a bookmark to edit or delete it, or right-click an empty area of the visible bookmarks bar to create and manage folders. If the bar is empty and hidden, create a folder from the address-bar bookmark editor. Right-click a folder to rename or delete it; deletions require confirmation.
- New installations start without sample bookmarks. Existing databases remove only the original DuckDuckGo, GitHub, Rust Lang, and Wikipedia starter entries once; other saved bookmarks are preserved.

## 8. Organized Browser Settings
- Settings are split into **Appearance & Layout**, **Search & Navigation**, **Workspaces & Theme**, **Performance & Privacy**, and **Keyboard Shortcuts**.
- Search all categories, settings, and keyboard shortcuts from the field beside the Browser Settings title; selecting a result opens its category and highlights the destination.
- Choose Google or DuckDuckGo as the default search engine in **Search & Navigation**. Google is selected by default; the choice applies to Omnibox and Command Bar searches and to newly opened tabs.
- Adjust the shared background blur for temporary overlays from **0–24 px** in Appearance & Layout; the preference is applied immediately and remembered across restarts.
- Show or hide the horizontal bookmarks bar in Appearance & Layout or with the toolbar toggle; the choice is remembered across restarts and does not affect bookmarks in the vertical sidebar. The bar appears only when at least one bookmark or folder exists.
- Use the fixed category rail to switch between panels; only the selected panel is shown and scrolls when its content is taller than the window.
- The last selected category is remembered when the dialog is reopened and across app restarts.
- In **Keyboard Shortcuts**, click a shortcut and press a key combination or Mouse 4/5 to record it. The workspace picker can use a held modifier key (Alt, Ctrl, Shift, or Meta); duplicate combinations are rejected. Changes are saved across restarts and work while the browser interface or a page has focus.
- Mouse 4 and Mouse 5 go back and forward by default and can be reassigned in **Keyboard Shortcuts**. Use `Ctrl + E` to focus and select the current address in the combined address and search bar.

## 9. Direct downloads

The Download Manager transfers a direct HTTP/HTTPS resource in a background worker. It writes to a temporary file, verifies completion, and saves without replacing an existing file before recording **Completed**. The panel shows transfer errors and prevents duplicate submissions while a transfer is running.

These URL downloads use an independent HTTP client without the page's authenticated cookie session, and save response bytes rather than a complete offline webpage archive. Downloads require the desktop app; browser preview reports an error instead of simulated success.
