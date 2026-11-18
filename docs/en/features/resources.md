# Resources panel

[Português](../../pt-BR/features/resources.md)

The **Resources** button (a gauge) is turned **off by default**. Turn it on in **Settings → Toolbar buttons** (the gear in the top bar). It opens a panel under the button with the browser's resources and its cleaning tools.

## What the panel shows

- **Memory**: the sum of what the open tabs use, with how many tabs are open and how many are sleeping (hibernated), and a **memory limit**.
- **Processor**: the share of the whole processor the pages are using, and a **processor limit**.
- **Internet**: the speed at which the pages are receiving, and a **speed limit**.
- **Cleaning**: opens the list of what can be cleared.

The numbers update every two seconds while the panel is open. They are measured only then, because measuring the network costs a little.

## Limits

Each limit has a row of choices; **None** turns it off. The choices are saved on this device.

| Limit | Choices | What it does |
| --- | --- | --- |
| **Memory** | 1, 2, 4 or 8 GB | When the open tabs use more than the limit, the tab you used longest ago is hibernated, one at a time, until they fit. The active tab, tabs with a form being edited, media playing, a download or picture-in-picture, and protected tabs are never hibernated. A hibernated tab comes back when you open it. |
| **Processor** | 25, 50 or 75% | The share of the whole processor the web engine may use, with the pages together. The browser's own interface is outside the cap, so it stays responsive; pages just run slower when they hit it. |
| **Internet** | 1, 5, 10 or 25 Mbps | The speed of the pages' network (download and upload) and of the browser's own downloads. |

### Per system

- **Windows**: the processor limit is a hard cap (a job object over the engine's processes, including the ones it starts later), and the internet limit slows the pages through the engine's developer protocol. The network measure also comes from it.
- **Linux**: the processor limit only **lowers the priority** of the web processes, so it matters when the computer is busy, and it is undone when the browser restarts (lowering the priority back needs privileges). The engine has no control over the pages' network, so the internet limit applies only to the **browser's own downloads**, and the network is not measured. The panel says so. Linux has not been tested yet.
- Downloads started by a page (not by the browser's download box) follow the internet limit on Windows only as far as the engine's network emulation reaches them.

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
