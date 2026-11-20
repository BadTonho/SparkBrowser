# Spark 0.1.4

## Fixes

- Fixed **Show rounded content corners**: with the option off, a border (an 8px gap) still remained around the page. Now the page fills the whole area when it is off.
- Fixed the settings tab (and the new-tab page) showing the wrong theme when **Dark mode on sites** was on: the browser was darkening its own pages, so the dark theme looked light and the light theme looked dark. The browser's own pages are now never changed by that option.
- Fixed the address bar suggestions staying on screen when you switched tabs or clicked elsewhere in the toolbar. The list now closes.
- Fixed keyboard shortcuts (F5 and the others) not working after clicking an empty area of the tab bar: the keyboard focus was lost. The shortcuts now keep working.
- Fixed F5 (and Ctrl+Shift+R) reloading the browser's own interface, which made the tabs flicker, instead of reloading the page. They now reload the active page.
