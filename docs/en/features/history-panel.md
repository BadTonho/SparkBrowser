# History panel

[Português](../../pt-BR/features/history-panel.md)

The **History** panel (toolbar button or shortcut) lists the pages you visited, newest first, over the left side of the window.

## Pop-up or tab

**Settings → Appearance & Layout → History display** (right below **Settings display**) chooses where History opens:

- **Pop-up** (default): the floating panel over the left side of the window, described below.
- **Tab**: History opens as a page of its own, `spark://history`, in a tab of the current workspace. If that workspace already has one, Spark switches to it instead of opening another. The page has the same cards, rows, menu and selection bar, centered and filling the tab, without the close button. The whole page scrolls, with a thin rounded scroll bar at the right edge of the tab, while the header (title, search and selection bar) stays at the top; opening a page or several pages keeps the history tab open, and clicking around it or pressing Escape never closes it. Typing `spark://history` in the address bar opens it too.

The choice is saved on this computer (like **Settings display**) and is not part of backups. Switching to **Tab** while the panel is open closes the panel. The browser preview (without the desktop app) always uses the pop-up.

## Layout

- **Header:** the title, a rounded search field (it searches titles and addresses) and **Clear all**.
- **One card per day**, headed "Today - Friday, November 27, 2026", "Yesterday - …" or the date alone, in the interface language.
- **One row per visit:** a check box, the time, the site's icon (its first letter when Spark has no icon for it), the page title and, after it in grey, the site (`youtube.com`). Hovering a row highlights it, and the title shows the full address as a tip.
- **A short vertical line** between two visits of the same day more than 30 minutes apart, so separate browsing sessions stand out.
- **Load more** at the end adds the next 60 visits.

## Actions

- **Click a title** to open the page in a new tab (the panel closes).
- **⋮ on a row** opens a menu: **More from this site** fills the search with that site, and **Delete from history** deletes that visit.
- **Check boxes** select visits; **Shift**-click selects the range from the last box checked. While something is selected, a bar replaces the header with the count, **✕** (clear the selection), **Open** (opens every selected page in its own tab) and **Delete** (deletes the selected visits). Clicking a title while something is selected checks it instead of opening it.
- **Escape** closes the row menu first, then clears the selection, then closes the panel. Arrow keys move inside the menu.

Deleting visits keeps the rest of the list as it is (the pages already loaded stay), and the new tab page and address suggestions stop showing them.

## How it works

- The panel is a native web view over the page (`public/history-overlay.html`, `public/overlays/history-overlay.js` and `.css`), painted by the shell (`HistoryDrawer.svelte`) with ready-made rows: `history-rows.ts` decides each visit's day, site, icon key and the pauses of more than half an hour. Site icons come from the icon cache as `data:image/` addresses, once per site.
- Actions return to Rust through the overlay bridge (`window/history_overlay/bridge.rs`), which accepts only `close`, `search`, `clear`, `load-more`, `open`, `open-many`, `delete` and `more-from-site`, and only for visits the panel shows. A selection holds at most 1,200 visits.
- `delete_history_entries` deletes visits by id in one transaction (at most 2,000 per call) and refreshes the new tab page. A failure is written to the error log and shown in the panel.
- The panel accepts up to 1,200 visits, so **Load more** keeps working past the first page.
- The tab is the same page with `?surface=tab` (`window/history_overlay/tab.rs`), served from the app's own assets and wired like the other internal pages (address, title, no zoom, never darkened, never saved as a visit). Like the settings tab, the shell sends it the rows (`set_history_tab`, when the tab is shown and again when the page finishes loading, `history-tab-loaded`), and the page sends typed actions back through the content bridge (`spark-internal://history-tab/action`), accepted only from the history document, for the session and tokens that received the rows, and only for the visits it shows. The `close` action is refused there. Sessions are forgotten when the tab navigates or closes.

Tests: `tests/history-panel.test.mjs` (days, pauses, registration, the selection bar and menu), the bridge tests in `bridge.rs` (open, open-many, delete and more-from-site only take shown visits) and `history_repo.rs` (deleting only the chosen visits).
