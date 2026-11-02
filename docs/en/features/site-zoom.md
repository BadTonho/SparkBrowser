# Persistent page zoom

The address bar shows the focused page's zoom and offers decrease, reset and increase controls. The default shortcuts are **Ctrl + +**, **Ctrl + -** and **Ctrl + 0**, including numpad equivalents; change their bindings in Settings. Levels range from 25% to 500%, with 100% as the default.

Zoom belongs to an HTTP/HTTPS origin: scheme, host and port. Workspaces share the setting. Changing it updates every resident page of that origin without recreating pages or moving focus. Hibernated tabs receive it when awakened. Split view controls follow the focused pane. Settings → Search and Navigation lists saved origins with individual reset actions. Preview shows that page zoom requires the desktop app.

Windows observers discard old WebView generations and navigation callbacks. Navigation applies the saved value rather than persisting the engine's automatic site defaults. Configured shortcuts run through the browser bridge; engine zoom hotkeys are disabled to prevent duplicate actions. See the [WebView2 controller reference](https://learn.microsoft.com/en-us/microsoft-edge/webview2/reference/win32/icorewebview2controller).

Backup exports use version 2 and include zoom preferences. Version 1 remains importable. Zoom is applied only through the durable preference task when **Apply backup preferences** is selected. Existing local origins win on conflicts; retrying the task does not overwrite them or import data again. Transfers, partial files and run markers remain excluded.

Validation: `node tools/testing/native-smoke.mjs --case zoom`, plus the existing `backup` scenario, against the isolated native build.
