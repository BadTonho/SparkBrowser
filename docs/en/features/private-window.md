# Private window

Open one with **Ctrl+Shift+N** or from the **⋯ More** menu → **New private window**. It has the same interface, settings, shortcuts and favorites as a normal window, and nothing it does is kept after it closes.

## How you can tell you are in one

The window has no native title bar, so Spark marks it itself:

- A purple **Private** badge with a crossed-out eye at the left of the toolbar, with a tooltip that says what is removed on close.
- A purple wash and a purple line along the bottom of the toolbar.
- On the new tab page, a notice (“You are in a private window”) and a purple background. It also states the limits: websites, your employer or your internet provider can still see what you do, and files you download stay on your computer.
- The operating-system window title reads “Spark (Private)”, which is what the taskbar and Alt+Tab show.

A private window does not offer **New private window** in its own menu.

## What it keeps

- Favorites (bookmarks and folders), site zoom, search shortcuts, new tab settings and your app preferences are copied when the window opens. Changes made inside the private window stay there.
- Browsing history, cookies, logins, local storage, cache, downloads records and open tabs are not copied and are removed when the window closes.
- Backup sync credentials are never copied.

## How it works

Each private window is a separate application process with its own disposable profile folder under the app's local data directory (`private-sessions`). When the window closes, its folder is removed. If the application crashes, the next start removes folders that are no longer in use.

## Implementation

- `get_private_window_state` tells the interface whether this process is a private window. The interface then marks the document (`data-private`), shows the badge (`PrivateBadge.svelte`) and styles the toolbar. The new tab page receives the same flag with its other data.
- Tests: the new tab notice is covered in `tests/new-tab.test.mjs`.

## Tradeoffs

- Favorites are a copy taken when the window opens. Edits do not sync back to the normal window.
- Each private window uses its own process, so it uses more memory.
- Extensions and logins from the normal profile are not available inside the private window.
- Files you download from a private window are saved where you choose and are not removed.
