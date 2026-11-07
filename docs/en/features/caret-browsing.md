# Caret browsing

Press **F7** to toggle caret browsing. A text caret appears in the active page so you can move through text with the keyboard and select it with **Shift** plus the arrow keys. The page engine handles links, frames and editable controls.

The mode starts off each time the app opens and applies to all content tabs for the current session. New tabs inherit the session state. Nothing is written to preferences or backups.

Windows uses WebView2's built-in caret browsing accelerator. Linux uses WebKitGTK's native `enable-caret-browsing` setting. The browser does not draw a simulated caret or inject page selection code. Exact caret movement and focus behavior follow the installed WebView engine.

Run `npm run test:native:build:fast`, then `npm run test:native -- --case caret-browsing` to check F7, selection, links, an embedded frame, editable fields, and state shared with new and existing tabs.
