# Ephemeral Tabs (Disposable Containers)

Veloce Browser provides disposable, isolated browsing tabs ("Ephemeral Tabs") designed for private, transient tasks that leave zero footprint on your machine.

---

## Capabilities

* **Isolated Temporary Storage:** Each ephemeral tab receives a dedicated, sandboxed WebView data directory inside `webview-ephemeral/<tab_id>`. Cookies, localStorage, sessionStorage, and HTTP cache remain isolated from your main workspaces and other tabs.
* **Instant Disk & Memory Purge:** When an ephemeral tab is closed, its data directory is immediately and completely purged from disk in a background worker thread.
* **Zero Session & History Persistence:**
  * Ephemeral tabs are excluded from the persistent SQLite `session_tabs` table and never restored when relaunching the browser.
  * Navigated pages inside ephemeral tabs are never written to the SQLite browsing history (`record_history_visit`).
  * Closed ephemeral tabs are omitted from the recently closed tabs history.
* **Visual Distinção:** Ephemeral tabs are tagged with a purple badge and stealth icon (`🕵️`), allowing you to easily identify disposable containers in both horizontal and vertical tab layouts.

---

## How to Open an Ephemeral Tab

1. **Horizontal Tab Bar:** Click the purple shield/mask button located next to the `+` New Tab button.
2. **Vertical Sidebar:** Click the **Ephemeral** pill in the sidebar tabs header.
3. Once opened, browse freely. Upon closing the tab, all session artifacts are destroyed permanently.

