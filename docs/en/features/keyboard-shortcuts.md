# Keyboard shortcuts across tabs

Keep **Ctrl** pressed and press/release **W** repeatedly to close consecutive active tabs. Each distinct W press closes one tab; holding W does not repeatedly close pages. **Ctrl + Shift + T** reopens recently closed tabs. These bindings remain configurable in Settings.

On Windows, native WebView accelerators read the current pressed state of modifiers independently of the previous page's message queue. Shortcut-driven closure transfers keyboard focus to the selected page after WebView reconciliation. It skips the transfer when an overlay or shutdown preparation is active, the selected tab has changed, or the window is minimized. Background closures and ordinary layout/resize reconciliation do not request focus.

The Windows `keyboard` native scenario sends real key input to the identity-verified test executable, closes three tabs while holding Ctrl, checks focus and repeat suppression, reopens three tabs while holding Ctrl+Shift, and closes from the shell. The input helper verifies the foreground executable before sending key-down events and releases injected keys in `finally`. Keep the test desktop available during this scenario. No extra application command or ACL permission is added for testing.
