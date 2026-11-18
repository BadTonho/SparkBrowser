# Color palettes

[Português](../../pt-BR/features/color-palettes.md)

The browser's accent colors can be changed in **Settings → Appearance & Layout → Color palette**. The accent is the color of the selected tab, the active buttons and switches, the highlights and the focus rings.

## Palettes

| Palette | Dark theme | Light theme |
| --- | --- | --- |
| **Graphite** (default) | light gray | near black |
| **Ocean** | blue | deeper blue |
| **Violet** | violet | deeper violet |
| **Emerald** | green | deeper green |
| **Rose** | rose | deeper rose |
| **Amber** | amber | deeper amber |

Graphite is the browser's original black and gray and is what a new installation uses. It adds nothing to the page: the other palettes only replace the accent colors, so the backgrounds, the text and the window style (Matte, Glass, Ice) stay as they are. Each palette has a lighter version for the dark theme and a deeper one for the light theme so the text on top of it stays readable.

## How it works

- The choice is saved on this device and is not part of backups.
- **It applies to every window at once**: windows that are already open change as soon as you pick a palette, with no need to reopen them.
- **It reaches the whole browser**: the panels that open as separate layers (tools, downloads, settings) copy the colors from the browser, and the new-tab page follows the palette too (focus rings, selected options, switches and sliders).
- **The shield button** shows its "on" state in the palette's accent. Graphite keeps the original green, so the default look does not change.
- **Colors with a meaning stay fixed**: the purple of private windows and ephemeral tabs, the green and red of the download and backup states, and errors.
- The palette is a single attribute on the page (`data-palette`); the colors are in [`src/styles/theme.css`](../../../src/styles/theme.css). To add a palette, add it to `PALETTES` in [`src/lib/services/palette-preference.js`](../../../src/lib/services/palette-preference.js), give it a dark and a light block in `theme.css`, a swatch in `settings.css` and a name in the language catalogs.
