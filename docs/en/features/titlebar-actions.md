# Reorder quick actions

Hold the left mouse button on a quick action at the right of the top bar for half a second, then drag horizontally. The highlighted button follows the pointer and a vertical marker shows the insertion position. Release inside the strip to save the order automatically for future launches.

A quick click still opens the action. Press **Esc**, switch away from the window, or release outside the strip to cancel. Moving more than 6 pixels before the half-second hold cancels the gesture and prevents an accidental click.

The order is shared between horizontal and vertical tab layouts. The bookmarks action keeps its saved position while hidden in vertical layout. The compact **More** menu keeps its existing order. Window controls stay fixed.

The preference is saved locally on this device and is not included in backup exports. If storage is unavailable, the new order remains usable for the current session.

## Choosing which buttons show

Only **Settings**, **Downloads**, **Protection** and **Extensions** are in the top bar by default. Every other button (Tools, Bookmarks bar, Commands, History, Zen mode, QR code, [Download video](video-download.md) and, on Windows, WhatsApp) is off until you turn it on. Open **Settings** (the gear) and use **Toolbar buttons** in the quick settings: each button is a toggle with its icon. A button you turn off keeps its slot, so it returns to the same place when turned on again. The Settings button cannot be hidden, because it opens this menu. From left to right the default order is Extensions, Protection, Downloads and Settings; buttons you turn on appear to the left of them, and you can still drag them anywhere. The choice is saved locally on this device and is not part of backups.

## Icon appearance

All quick actions use decorative outline SVGs, sized at 16 × 16 pixels with a consistent 1.8-unit stroke. Tools uses a wrench, bookmarks a star, commands a lightning bolt, history a clock, downloads a downward arrow, privacy a shield, settings a gear, and focus four framing corners.

The icons inherit the button's CSS `color` through `stroke="currentColor"`. Change the button's `color` or the existing theme variables (`--text-secondary`, `--text-primary` for hover, and `--accent-primary` for active bookmarks) to recolor them. The green privacy status indicator remains separate. Icon colors do not have a settings picker.


## QR Code

The QR Code quick action opens a local panel for the active HTTP(S) page. It generates a PNG without a network service and lets you copy the image or save it through the native file dialog. If image copying fails, the panel reports the error and keeps the save action available. Internal pages and URLs beyond QR capacity show an explanation. The action participates in the saved, reorderable titlebar strip.