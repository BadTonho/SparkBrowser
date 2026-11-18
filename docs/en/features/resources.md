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

Each limit is a slider you drag to the value you want, with the chosen value shown centered above it. You can also **click the number and type an exact value** (memory in GB, processor in %, internet in Mbps) and press Enter. Typed values are kept inside what the limit allows: memory 0.25 to 128 GB, processor 1 to 95%, internet 0.1 to 1000 Mbps. Drag the slider all the way to the right, or clear the number, for **No limit**. The limit is applied when you release the slider or confirm the number, and it is saved on this device.

| Limit | Range | What it does |
| --- | --- | --- |
| **Memory** | 512 MB to 16 GB, in steps of 512 MB | The tab you are on is never touched. When the open tabs use more than the limit, the others are hibernated, the ones you used longest ago first, as many as it takes to fit (checked every few seconds, right away when you change the limit or switch tabs). Tabs with a form being edited, media playing, a download or picture-in-picture, and pinned or protected tabs are not hibernated; while they are in the background they are asked to use as little memory as the engine can give back (Windows only), and they go back to normal when you open them. A hibernated tab comes back when you open it. If the tab you are on alone uses more than the limit, nothing else can be done: the limit never closes or slows the tab you are using. |
| **Processor** | 10 to 90%, in steps of 5 | The share of the whole processor the web engine may use, with the pages together. The browser's own interface is outside the cap, so it stays responsive; pages just run slower when they hit it. |
| **Internet** | 1 to 1000 Mbps: steps of 1 up to 10, of 5 up to 50, of 10 up to 100, then of 100 | The speed of the pages' network (download and upload) and of the browser's own downloads. |

### Per system

- **Windows**: the processor limit is a hard cap (a job object over the engine's processes, including the ones it starts later), and the internet limit slows the pages through the engine's developer protocol. The network measure also comes from it.
- **Linux**: the processor limit only **lowers the priority** of the web processes, so it matters when the computer is busy, and it is undone when the browser restarts (lowering the priority back needs privileges). The engine has no control over the pages' network, so the internet limit applies only to the **browser's own downloads**, and the network is not measured. Linux has not been tested yet.
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
