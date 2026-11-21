# Spark 0.1.4

## New features

- **HTTPS-only mode** (Settings → Privacy & Security → Network privacy): opens the secure version of every site. If it cannot be reached, a notice asks before continuing over HTTP, once and only for that address. Local addresses such as localhost are not changed. It is off by default.
- **DNS over HTTPS** (same place): looks up site names through Cloudflare or Google so the network cannot read or change them, with no fall back to the ordinary DNS. It is off by default and applies after a restart. Video calls (WebRTC) and the browser's own update and icon requests are not covered.
- **Background media can now be repositioned**: drag the preview in Settings → Background → Background media to choose which part of the image or video stays visible on the new-tab page.
- The **background media can now be zoomed**: a new Zoom slider (25% to 300%, so it can also shrink the picture) in Settings → Background → Background media changes how much of the image or video is shown: 100% fills the screen, lower shows more of it (down to all of it, with the background around), higher enlarges it from the center. The preview now has the shape of your screen, so it shows the real framing.

## Improvements

- The **Tor session** entry is no longer shown in the privacy settings: it is postponed until traffic leaks can be ruled out. Nothing about it was ever active.
- The **About & Updates** page now shows the link to the browser repository on GitHub, with a button that opens it in a new tab.
- The **audio equalizer intensity** now changes the size of the effect itself (bars, rings, orbs, particles and so on grow or shrink), not just how visible it is. The range went from 0.5–2 to 0.25–4 and the effect follows the slider while you drag it. A new **Effect opacity** slider next to it controls how transparent the effect is. Both sliders now show their value as a percentage (for the intensity, 100% is where the slider used to end and it now goes up to 200%).

## Fixes

- Fixed **Show rounded content corners**: with the option off, a border (an 8px gap) still remained around the page. Now the page fills the whole area when it is off.
- Fixed the settings tab (and the new-tab page) showing the wrong theme when **Dark mode on sites** was on: the browser was darkening its own pages, so the dark theme looked light and the light theme looked dark. The browser's own pages are now never changed by that option.
- Fixed the address bar suggestions staying on screen when you switched tabs or clicked elsewhere in the toolbar. The list now closes.
- Fixed keyboard shortcuts (F5 and the others) not working after clicking an empty area of the tab bar: the keyboard focus was lost. The shortcuts now keep working.
- Fixed F5 (and Ctrl+Shift+R) reloading the browser's own interface, which made the tabs flicker, instead of reloading the page. They now reload the active page.
- Fixed the "Close the browser?" dialog appearing with nothing to save: the backup made when closing is now built only when something changed since the last backup, so closing with nothing new is instant.
- Fixed the mouse back/forward side buttons (4 and 5) going more than one page at a time: the browser was also navigating by itself when the button was released. Now they move exactly one page.
- Fixed the YouTube card on the new tab: its progress bar could not be clicked or dragged. It now jumps to the moment you click or drag to, in the tab that is playing.
- Fixed the YouTube card on the new tab doing nothing when the tab it controls was hibernated. Pressing play (or any control) now wakes that tab in the background, reloads its page and starts the video, without leaving the new tab.
