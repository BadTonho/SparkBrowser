# Command Bar appearance

[Português](../../pt-BR/features/command-bar.md)

The Command Bar (**Ctrl + K** by default) opens as a layer above the page. **The page stays visible behind it**, dimmed, instead of being replaced by a dark screen, so you keep your place while you search or run a command. What it searches and runs is described in [Notes and sessions in the Command Bar](./command-search.md).

## How it looks

- It uses the same surface, border and shadow as the other browser panels, so it follows the theme (dark or light), the window background style (Matte, Glass, Ice) and the color palette.
- It is clean: a single input row, a list with one quiet tint on the selected row (no outline, glow or sliding), small neutral badges and a one-line footer with the keys.
- The text field fits its placeholder ("Search, go to an address or run a command").

## How it works

- Inside the app the bar is drawn by a native layer above the page. The real bar stays in the browser window, off screen, and keeps all the logic; the layer shows a copy of it and forwards what you type, the arrow keys, **Enter** and clicks to it.
- **Esc**, **Ctrl + K**, a click outside the bar or a switch to another window close it. When it closes the keyboard goes back to the browser window.
- The layer is prepared a few seconds after the browser starts and kept for three minutes after each use, so opening it with the shortcut is quick.
- Outside the app (the interface previewed in a plain browser) there is no native layer, so the bar is drawn in the page as before.
