# Spark 0.1.3

## New features

- The window background style now has three options: **Matte** (solid), **Glass** (clear and light-catching, with a sheen and a bright edge) and **Ice** (a strong blur). The earlier "Frosted Glass" choice is now called **Ice** and is kept if you had it selected.

- Clicking the address bar or typing in it now shows suggestions below it: your most recent sites when it is empty, and, while you type, what Enter will do, then matching bookmarks and visited pages. Use the arrow keys and Enter, or click a row.

- New option **Search suggestions** (Settings → Search & Navigation, on by default): while you type in the address bar, it asks your search engine for suggested searches and lists them under the matching sites. It works with Google, DuckDuckGo, Bing and Brave Search, sends only what you typed, and is never used in private windows.

- You can now change the browser's color palette in **Settings → Appearance & Layout**: Graphite (the original black and gray, and the default), Ocean, Violet, Emerald, Rose and Amber, each with a version for the dark and the light theme. It applies to every open window at once, to the new-tab page and to the shield button.

- The Command Bar (Ctrl + K) now opens above the page, which stays visible, dimmed, behind it instead of being replaced by a dark screen with a placeholder card. It was redesigned to be cleaner and to match the rest of the browser (same surface as the other panels, a quiet selected row, no glow), and its text field now fits its placeholder.

## Improvements

- In the centered customization panel, the **Panel position** and **Music widgets position** options are now small, sit side by side and stay at the bottom of the panel, leaving more room for the cards.

- The labels of the new-tab customization options are shorter: **Compact** and **Center** for the panel position, **Default** and **Centered** for the music widgets position.

- The new-tab page adapts better to the window width: the Weather and Gmail cards shrink to fit beside the search box instead of dropping above it, which keeps the search box and recent sites in view on medium-sized windows. A tall Gmail card now scrolls inside itself.

- In **Music and video**, Spotify and YouTube are now a vertical list with their logos, instead of two side-by-side tabs. In the centered panel the list sits on the left and the options of the chosen service on the right. Use the up and down arrow keys to move between them.

- In **Wallpaper**, the audio equalizer and the background media are now a vertical list too. In the centered panel the list sits on the left and the options of the chosen group on the right, so the page no longer needs a long scroll.

- The audio equalizer style is now chosen from a grid of small previews, each drawn in the color you picked, instead of a dropdown list.
- In the centered panel, the **Background media** preview is much larger.

- The "Close the browser?" window has a cleaner look: more spacing, a lighter "Don't show again" option and clearer buttons.

- In **Clock**, the font, time format and position are now chosen from visible buttons instead of dropdown lists. Each font is shown with a sample time in that font.

- The browser settings menu has a new order: Appearance & Layout, Search & Navigation, Workspaces & Theme, Performance, Privacy & Security, Keyboard Shortcuts, About & Updates, Changelog, Local Diagnostics and Data & Backup.

- In the browser settings, the option selectors are easier to read: each group of choices (such as Pop-up / Tab, Dark / Light) is a clear segmented control with the chosen option filled in, and titles and descriptions are larger.

- In the browser settings, every on/off option is now the same switch, instead of a mix of switches and blue squares (for example in Performance and Privacy & Security).

- In the browser settings, the title is now just **Settings** and the search is a single icon: click it, or anywhere in the free space beside it, and it opens into the search field. It closes again when you leave it empty.

- When the browser stops responding, the error log now lists what it was doing just before (the last commands and panels) and, on Windows, where every part of the app is waiting, so the cause can be found instead of only seeing that it froze.

- Every failed command and every console error of the interface is now written to the error log, and the log follows four levels (Critical, Error, Warning, Info).

- The top bar now shows only Settings, Downloads, Protection and Extensions by default. Turn the other buttons (Tools, Bookmarks bar, Commands, History, Zen mode, QR code, WhatsApp) on or off in **Toolbar buttons**, inside the quick settings menu.

- Adblock now also hides the ads and "sponsored" boxes that the filter lists name on each page, not only blocks their requests. YouTube ads are removed before the player receives them, and "Auto skip ads" now also plays an ad that has no skip button to its end.

- Dragging a tab now shows the tab itself following the pointer (with its icon and title), leaves a dashed placeholder in its place, and the other tabs slide smoothly to their new positions when you reorder them.

- Opening a split is now smooth: the pages slide to their new places like the tabs do, and the area where a dragged tab opens a split is much larger (15% of the page width, instead of a thin strip at the edge).

- The settings tab opens faster: it no longer shows a white page while it loads (it is dark from the first moment) and the shell notices that the page is ready sooner.

## Fixes

- Fixed choosing an image or video as the new-tab background having no effect in the installed app.
- Fixed the "Close the browser?" warning appearing because of the new tab or settings page: changing an option there was counted as an edited form, and "Don't show again" could never apply.
- Fixed the window background style (Matte, Glass, Ice) not updating the windows right away; you had to move the opacity or blur slider to see the change.
- Fixed the on/off switches in the browser settings (such as Hardware acceleration) looking the same when on and when off; the knob was white on a light track. It is now clearly visible, in the dark and light themes.
- Fixed the browser freezing for good when a new tab was replaced (for example by opening the settings tab) while the audio visualizer was running.
- The error log of a freeze now lists the main thread and the threads running Spark's own code first, so the cause is not cut off.
