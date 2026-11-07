# Navigation interactions

Zen mode defaults to Ctrl+Shift+Z. A saved Alt+Z default migrates when the new combination is free; other assignments are retained and a conflict prevents that migration.

The native link context menu routes a selected HTTP(S) link to a background tab in the source workspace, with its container and ephemeral status. Middle-click and Ctrl-click use a trusted gesture on a link in the top-level document. Automatic pop-ups remain governed by the pop-up setting. Links inside frames and gestures intercepted by a page require separate native validation.

WebView fullscreen requests expand the browser window and the requesting content surface. Leaving fullscreen restores the previous window mode; selecting or closing the requesting tab exits it. The browser does not grant background tabs control over the window.

The content viewport uses the frame's inner bounds so the frame border does not overlap the page's vertical scrollbar. Test split layouts, resizing and native hit testing on each supported platform when changing this geometry.

The `navigation-corrections` native scenario uses loopback pages and an isolated profile. It passed on Windows for Ctrl-click, middle-click, automatic pop-up blocking, Escape, the persisted history setting and shortcut migration with another custom binding. The Windows `fullscreen-corrections` scenario passed for Escape and changing tabs, and `new-tab-widgets` passed for settings, focus, narrow layouts and split view. Native context-menu and scrollbar-edge checks, additional fullscreen restoration cases, and WebKitGTK execution remain to be verified.
