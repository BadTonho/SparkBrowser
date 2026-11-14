# Several browser windows

Open another window with **Ctrl+N** or from the **⋯ More** menu → **New window**. Each window has its own tab bar, address bar, selected tab, selected workspace and split view, so one can sit on each monitor. Cookies, logins, favorites, history, downloads, settings and workspaces are shared by every window.

This is different from a [private window](./private-window.md): it is a separate process that keeps nothing. Normal windows share one profile.

## Moving tabs between windows

Right-click a tab and choose **Move to window N** or **Move to a new window**. The tab becomes the selected one in the destination; the window it left selects a neighbour (or a new-tab page when that was its last tab). The page reloads in its new window.

## Closing

- Closing the **first window** closes the browser, with the usual [protected shutdown](./shutdown.md) review. The review covers the pages of every window.
- Closing **another window** never deletes its tabs: they go to [recently closed tabs](./closed-tabs.md), where they can be reopened.
- Closing the last tab of a window leaves a new-tab page, as in the first window.

## What comes back on the next launch

Spark saves every window with its tabs, the tab and workspace it had selected, and where it was on screen (size, position, maximized and monitor). On the next launch **all windows reopen**, also after a crash or a forced kill. This protects tabs that live in a window other than the first one.

- Windows return to the same position on Windows and on Linux with X11. On **Wayland** the compositor chooses where a window goes, so Spark restores the tabs, the size and the maximized state but not the monitor.
- A position that no monitor shows any more (a screen that was unplugged) is ignored, so a window never reopens out of reach.
- If a window cannot reopen, its tabs go to recently closed tabs and the browser reports the error. Nothing is dropped silently.

## Panels and menus

Popovers (tools, downloads, settings, history, privacy, permissions, zoom, media), the workspace picker and the bookmark menu open inside the window that asked for them. Fullscreen video fills only its own window.

## Tests

```
node tools/testing/native-smoke.mjs --case browser-windows
node tools/testing/native-smoke.mjs --case window-restore
```

`browser-windows` opens a second window, checks that tabs stay separate, moves a tab, prepares panels in both windows and closes the window. `window-restore` restarts the app, first by killing it and then with a normal exit, and checks that both windows return with their tabs.
