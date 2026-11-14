# Ephemeral Tabs (Disposable Containers)

Spark Browser provides disposable, isolated browsing tabs ("Ephemeral Tabs") designed for private, transient browsing tasks.

---

## Capabilities

* **Isolated Temporary Storage:** Each ephemeral tab receives a dedicated, sandboxed WebView data directory inside `webview-ephemeral/<tab_id>`. Cookies, localStorage, sessionStorage, and HTTP cache remain isolated from your main workspaces and other tabs.
* **Temporary data cleanup:** Closing a tab schedules deletion of its data directory, with retries if the engine still holds files. Remaining data is removed synchronously after the profile lock is acquired on the next startup. If cleanup fails at startup, the browser reports an error rather than reusing the directory.
* **Zero Session & History Persistence:**
  * Ephemeral tabs are excluded from the persistent SQLite `session_tabs` table and never restored when relaunching the browser.
  * Navigated pages inside ephemeral tabs are never written to the SQLite browsing history (`record_history_visit`).
  * Closed ephemeral tabs are omitted from the recently closed tabs history.
* **Visual Distinção:** Ephemeral tabs are tagged with a purple badge and stealth icon (`🕵️`), allowing you to easily identify disposable containers in both horizontal and vertical tab layouts.

---

## How to Open an Ephemeral Tab

1. **Horizontal Tab Bar:** Click the purple shield/mask button located next to the `+` New Tab button.
2. **Vertical Sidebar:** Click the **Ephemeral** pill in the sidebar tabs header.
3. Once opened, browse freely. Closing the tab schedules cleanup of its temporary WebView data. Downloads and data saved by websites outside that directory are not erased.

