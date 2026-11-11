# Private window

Press **Ctrl+Shift+N** to open a private window. It looks and behaves like a normal window: same interface, settings, shortcuts and favorites. Nothing it does is kept after it closes.

## What it keeps

- Favorites (bookmarks and folders), site zoom, search shortcuts, new tab settings and your app preferences are copied when the window opens. Changes made inside the private window stay there.
- Browsing history, cookies, logins, local storage, cache, downloads records and open tabs are not copied and are removed when the window closes.
- Backup sync credentials are never copied.

## How it works

Each private window is a separate application process with its own disposable profile folder under the app's local data directory (`private-sessions`). When the window closes, its folder is removed. If the application crashes, the next start removes folders that are no longer in use.

## Tradeoffs

- Favorites are a copy taken when the window opens. Edits do not sync back to the normal window.
- Each private window uses its own process, so it uses more memory.
- Extensions and logins from the normal profile are not available inside the private window.
- Files you download from a private window are saved where you choose and are not removed.
