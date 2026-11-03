# Keyboard shortcuts across tabs

Keep **Ctrl** pressed and press/release **W** repeatedly to close consecutive active tabs. Each distinct W press closes one tab; holding W does not repeatedly close pages. **Ctrl + Shift + T** reopens recently closed tabs. These bindings remain configurable in Settings.

**Ctrl+Tab** and **Ctrl+Shift+Tab** select the next/previous tab, wrapping within the focused split group or the whole workspace outside split. Numbered selection (Ctrl+1–9, with 9 selecting the last tab), new tabs (Ctrl+T) and tab reordering also target the focused group. Selecting a tab from the unified vertical list focuses its owning pane. See [split view](split-view.md).

In a compact horizontal split header, focus **More** and press Enter, Space or Down to open its menu. Up/Down cycle through actions, Home/End select the first/last action, and Enter/Space activate it. Escape or Tab closes the menu and returns focus to More. Dialogs opened from the menu also return focus to that button when dismissed.

On Windows, native WebView accelerators read the current pressed state of modifiers independently of the previous page's message queue. Shortcut-driven closure transfers keyboard focus to the selected page after WebView reconciliation. It skips the transfer when an overlay or shutdown preparation is active, the selected tab has changed, or the window is minimized. Background closures and ordinary layout/resize reconciliation do not request focus.

The Windows `keyboard` native scenario sends real key input to the identity-verified test executable, closes three tabs while holding Ctrl, checks focus and repeat suppression, reopens three tabs while holding Ctrl+Shift, and closes from the shell. The input helper verifies the foreground executable before sending key-down events and releases injected keys in `finally`. Keep the test desktop available during this scenario. No extra application command or ACL permission is added for testing.
