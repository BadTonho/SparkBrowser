# YouTube now-playing widget

[Português](../../pt-BR/features/youtube-widget.md)

Open **Customize new tab → Music and video → YouTube** and enable **Show YouTube now playing**. The card appears below Weather and Spotify on the left. **Music and video** groups Spotify and YouTube as two tabs; switch between them with a click or the left and right arrow keys.

The card is deliberately minimal. Its heading is just the YouTube logo (the YouTube Music logo for music.youtube.com); clicking it goes to the video's tab. Below it are the thumbnail, the title on a single line, a thin progress bar with the elapsed and total time, and previous, play/pause and next controls. A title that does not fit scrolls slowly from side to side, pausing at each end; with reduced motion enabled it is shortened with an ellipsis instead. Live streams have no length, so the bar is hidden for them. When several YouTube tabs are open, a tab that is playing wins, then the active tab, then the most recently used one. With no YouTube video open, the card explains how to start one.

## Thumbnail and preview options

Under **Customize new tab → Music and video → YouTube**:

- **Thumbnail:** **Small, beside the title** (default) or **Cover the whole widget**, which fills the card with the thumbnail behind a dark gradient so the title, bar and controls stay readable.
- **Preview:** **Thumbnail only** (default) or **Muted video**, which plays the video itself in the thumbnail slot, or across the whole card in cover mode.

These two choices are local to the device and are shared by all new tabs. They are not part of the account-style widget settings and are excluded from backups.

Choose either option by clicking its example card or by using Tab and the arrow keys. The Preview examples follow the selected thumbnail layout. Hover over or focus the **Muted video** example to see it move if reduced motion is off. Examples are local and available even when no YouTube video is open.

The muted video is a second, muted copy of the video loaded in YouTube's privacy-enhanced player (`youtube-nocookie.com`) inside the card; the tab you are watching keeps the sound. It follows the tab: it starts at the tab's position, pauses and resumes with it and jumps when you seek by more than a few seconds. It cannot be clicked, never shows controls, and stops while the new tab is hidden. The player is zoomed in so YouTube's own title bar and logo fall outside the card, and it is shown only while the video is really playing: while it loads, when you pause the tab, or when the video refuses embedding, the thumbnail is shown instead. It uses extra data and can show ads, and videos that disallow embedding fall back to the thumbnail. It is the only case where the new-tab page contacts YouTube: the page's policy allows frames from that single host (`frame-src https://www.youtube-nocookie.com`, also listed in the app's `tauri.conf.json`) and still blocks every other connection.

## In the vertical tabs sidebar

With the vertical tab layout, the same card also appears at the bottom of the sidebar, under the tab list, whenever a YouTube or YouTube Music video is open. It is a compact version: the logo (click it to go to the tab), the thumbnail, the one-line scrolling title, the progress bar with the elapsed and total time, and previous, play/pause and next controls. It follows the same **Thumbnail** choice as the new-tab card (small, or covering the whole card), but never plays the muted video preview. It disappears when no YouTube video is open.

It needs the widget to be on (**Show YouTube now playing**). Turn off **Also show in the vertical tabs sidebar** in the same settings to keep the card on the new tab only. Because the sidebar is always on screen, Spark asks the video tab for its position every few seconds while this option is on, even when no new tab is visible. That request is the same local media report described below; it adds no network traffic. The option is saved with the widget settings and, like them, stays on the device and out of backups.

The illustration below the switch shows where the widget sits and follows the selected thumbnail layout. It stays visible when the sidebar switch is off.

## How it works

No account, API key or sign-in is needed. Spark reads only what it already tracks for each tab: its address, title and whether it is playing. A tab counts when it is an `https` watch page (`/watch?v=…` with an 11-character video ID) on `youtube.com`, `www.youtube.com`, `m.youtube.com` or `music.youtube.com`. The title has YouTube's notification count and site suffix removed.

Controls act on the tab itself: play/pause toggles the page's video element, and next and previous press the player's own buttons. Previous restarts the video when the player has no previous button. If the tab was closed or hibernated, the card shows an error and refreshes on its own.

## Refresh and privacy

The card asks the video tab for a fresh position every few seconds while a normal new tab is visible, and about one second after a control; between samples the bar advances on its own while the video plays, so a seek shows up at the next sample. The tab answers through the same fixed, token-checked media report it already uses for playback state, now carrying the position and length in seconds; reports with invalid numbers are ignored. Updates stop while the widget is disabled or the page is hidden. Nothing is stored. The setting is device-local and excluded from backups.

Unless the muted video is turned on, the only network request is the public thumbnail, `https://i.ytimg.com/vi/<id>/mqdefault.jpg`, fetched by Rust with a size limit and JPEG signature check, then passed to the page as image data. This keeps the new-tab page's `connect-src 'none'` policy intact. Private windows never read tabs or show the widget.

## Validation

Unit tests cover URL and title parsing, rejected hosts, ports, credentials and IDs, the per-source control scripts, rendering, thumbnail filtering, the tab switcher and private-window hiding. Real playback and tab controls need a manual check with a YouTube page.
