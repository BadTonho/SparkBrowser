# Resources panel

[Português](../../pt-BR/features/resources.md)

The **Resources** button (a gauge) is turned **off by default**. Turn it on in **Settings → Toolbar buttons** (the gear in the top bar). It opens a panel under the button with the browser's resources and its cleaning tools.

## What the panel shows

- **Memory in use**: the sum of what the open tabs use, with how many tabs are open and how many are sleeping (hibernated). It updates while the panel is open.
- **Cleaning**: opens the list of what can be cleared.

Limits for memory, processor and internet are the next step and are not in this version of the panel yet.

## Cleaning

Choose what to clear and press **Clear selected**; the button asks for a second click to confirm, because none of this can be undone.

| Item | What it clears |
| --- | --- |
| **Cache** | Saved copies of pages and files. Sites may load a little slower once. |
| **Cookies and site data** | Cookies, local storage, databases and service workers that sites keep. You are signed out of the sites you are logged in to. |
| **Browsing history** | The browser's history of visited pages. |
| **Download list** | Finished downloads leave the list; the files stay on the disk. |

Only the cache is chosen when the panel opens. The result is shown in the panel: what was cleared, or what could not be.

## Good to know

- The cache and the site data are cleared in every data store reachable from an open page. Workspaces with isolated data have a store of their own; the store of a workspace whose tabs are all hibernated is not reached until one of them wakes up.
- The browser's own settings (theme, color palette, toolbar buttons) live in the same storage as the sites' when the data is shared. They are saved before the site data is cleared and written back afterwards, so clearing site data does not reset the browser.
- Windows uses the WebView2 profile (`ClearBrowsingData`) and Linux uses the WebKitGTK data manager. Linux has not been tested yet.
