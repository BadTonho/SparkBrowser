# Tab diagnostics

Open **Tools → Tab diagnostics**, or search for it in the Command Bar. Filter by the current workspace or all workspaces and search titles and URLs. Totals describe the selected workspace scope; the configured resident limit remains global and soft.

Each row reports actual WebView residency, time since activation, workspace and all protection reasons: selection, split pane, pinning, loading, edited form, media and downloads. Paused native downloads continue to protect the WebView they depend on. These reasons use the same backend policy as hibernation and reviewed bulk closing.

Select a page or hibernate an unprotected resident. The backend rechecks protection at execution time. Data refreshes every two seconds only while the panel is mounted; obsolete replies are discarded. Preview indicates that native diagnostics require the desktop app. WebView counts do not measure process memory.

Validation: `node tools/testing/native-smoke.mjs --case tab-diagnostics` with the isolated native build.
