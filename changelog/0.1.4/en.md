# Spark 0.1.4

## New features

- **HTTPS-only mode** (Settings → Privacy & Security → Network privacy): opens the secure version of every site. If it cannot be reached, a notice asks before continuing over HTTP, once and only for that address. Local addresses such as localhost are not changed. It is off by default.
- **DNS over HTTPS** (same place): looks up site names through Cloudflare or Google so the network cannot read or change them, with no fall back to the ordinary DNS. It is off by default and applies after a restart. Video calls (WebRTC) and the browser's own update and icon requests are not covered.

## Improvements

- The **Tor session** entry is no longer shown in the privacy settings: it is postponed until traffic leaks can be ruled out. Nothing about it was ever active.

## Fixes

- Fixed **Show rounded content corners**: with the option off, a border (an 8px gap) still remained around the page. Now the page fills the whole area when it is off.
- Fixed the settings tab (and the new-tab page) showing the wrong theme when **Dark mode on sites** was on: the browser was darkening its own pages, so the dark theme looked light and the light theme looked dark. The browser's own pages are now never changed by that option.
- Fixed the address bar suggestions staying on screen when you switched tabs or clicked elsewhere in the toolbar. The list now closes.
- Fixed keyboard shortcuts (F5 and the others) not working after clicking an empty area of the tab bar: the keyboard focus was lost. The shortcuts now keep working.
- Fixed F5 (and Ctrl+Shift+R) reloading the browser's own interface, which made the tabs flicker, instead of reloading the page. They now reload the active page.
- Fixed the "Close the browser?" dialog appearing with nothing to save: the backup made when closing is now built only when something changed since the last backup, so closing with nothing new is instant.
