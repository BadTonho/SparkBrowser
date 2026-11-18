# Address bar suggestions

Clicking the address bar, or typing in it, shows a list of suggestions right below it.

## What the list shows

- **Just focused (nothing typed):** the sites visited most recently, one row per address, so it is one click to go back.
- **While typing:** the first row is always what **Enter** does with the text as typed: *Search with* your default engine for words, or *Go to address* when the text looks like one (`example.com`, `localhost:1420`, `https://…`). Below it come your **bookmarks** that match, then **visited pages** from the history that match, best first.
- A page is listed once, whatever its scheme, `www.` or trailing slash. Every typed word has to appear in the title or the address.
- Pages that start with what you typed and pages you visit often rank higher; recent visits count a little more.

## Using it

| Key / action | Result |
| --- | --- |
| Up / Down arrow | Move the highlight; past either end it returns to "nothing highlighted" |
| Enter | Open the highlighted row, or do what the typed text asks when none is highlighted |
| Esc | Close the list and restore the address bar |
| Click a row | Open it; the address bar keeps focus until the page starts loading |

The list lines up with the address bar. It sits in the page layout, so the page below moves down while the list is open; the list only grows during one session, so the page does not jump on every key.

## Notes

- Everything comes from the local history and bookmarks; nothing is sent anywhere. See [Privacy & Security settings](privacy-security-settings.md) for the history retention.
- The history is searched with the same query as the History panel, over the latest 60 matching visits.
