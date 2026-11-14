# Hardware acceleration

Settings → Performance has a **Hardware acceleration** switch. It is **on by default**: pages and video are drawn with your graphics card, which is faster and uses less CPU. Turn it off if you see blank or white windows, flicker, tearing or other graphics glitches, which some drivers and virtual machines cause. Pages then render in software: everything works, but heavy pages and video use more CPU.

## Applying a change

The web engine reads its graphics options only once, when the browser process starts, so a change takes effect after a restart:

1. Flip the switch. The choice is saved immediately.
2. A note appears (“Restart Spark to apply this change.”) with a **Restart now** button.
3. **Restart now** uses the same protected shutdown as a WebView2 runtime update: pending notes and the session are saved first, then Spark starts again with the new setting. You can also just close and reopen Spark later.

If you flip the switch back before restarting, the note disappears: the setting is compared with what the running process actually started with.

## How it works

- The choice lives in `graphics.json` inside the app data folder, not in the database or the interface, because it has to be read before any web view exists.
- On Windows, when acceleration is off, Spark adds `--disable-gpu` to `WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS`; arguments you set yourself in that variable are kept. On Linux it sets `WEBKIT_DISABLE_COMPOSITING_MODE=1` and `WEBKIT_DISABLE_DMABUF_RENDERER=1` (WebKitGTK), unless you already set them.
- Private windows and restarted processes inherit what the main process configured. A restarted process removes only the flags Spark itself added, tracked with an internal marker (`SPARK_GRAPHICS_FLAGS`), so turning acceleration back on never strips something you set.
- A missing, unreadable or damaged `graphics.json` means acceleration stays **on**. A byte order mark at the start of the file, which Notepad and Windows PowerShell add when saving UTF-8, is ignored, so a hand-edited file works.

## Not part of backups

GPU problems belong to one machine and one driver, so this setting is not included in backups, sync files or imported preferences. Restoring a backup on another computer never turns acceleration off there.

## Limits

- The switch controls the web view's GPU use only. Operating-system or driver settings (for example per-app GPU selection in Windows) are separate.
- In the browser preview used during development there is no GPU choice to make, so the switch reports acceleration as on and never asks for a restart.

## Implementation

- Native: `src-tauri/src/graphics.rs` (file, environment, status) and `commands/graphics.rs` (`get_graphics_status`, `set_hardware_acceleration`, `restart_for_graphics`). The flags are set in `run()` before the first web view exists.
- Interface: `graphicsStore.ts` and the row in `PerformanceSettings.svelte`.
- Tests: unit tests in `graphics.rs` cover the default, damaged files, a file with a byte order mark, argument merging and the environment round trip. The effect on a real WebView2 was checked by launching the native test build with the setting off and reading the browser process command line, which then carries `--disable-gpu`.
