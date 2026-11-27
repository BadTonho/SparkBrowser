# Tab hover card

Resting the mouse on a tab for about half a second opens a card next to it with:

- the tab name, shortened to one line (at most 60 characters);
- a picture of the page;
- the memory the tab uses, or the memory it saves when it is asleep.

The card opens below the tab with horizontal tabs and to its right with vertical tabs, and always stays inside the window. Once a card is open, moving to the next tab shows that tab's card at once. Pressing a tab, scrolling, starting a drag, opening the tab menu, leaving the tab or switching to another app closes it. The card never takes focus or the mouse, so the tabs below it can still be clicked. It replaces the text tip the tabs showed before.

## Page pictures

Only a page that is on screen can be pictured, so Spark takes the picture:

- of the page you are viewing, about 1.5 seconds after you switch to it;
- of the pages on screen while the mouse is over the tabs (at most every 3 seconds), so the tab you are about to leave keeps a recent picture;
- of the hovered tab, when it is on screen (the card first shows the last picture and then updates).

A tab in the background or asleep shows how its page looked the last time it was on screen. A tab never seen in this run shows "No preview yet" instead.

Pictures are small (528 pixels wide; JPEG on Windows, PNG on Linux) and kept only in memory: they are never written to disk, the session or backups. Other tabs are never loaded or woken up to take a picture. Spark keeps at most 40 of them, drops the least recently used ones first, and forgets the picture of a closed tab.

## How it works

Pages are separate web views that cover the browser interface, so a card drawn by the interface would be hidden behind the page. Like the tab drag preview, the card is drawn by its own transparent window (`tab-hover-card`). That window is only as large as the card, sits next to the tab, ignores the mouse and is shown without being activated. Moving from one tab to the next keeps it on screen and only changes its content and position. When the card closes, its content is cleared at once, the window is hidden half a second later (so it reopens from an empty frame, never with the previous card) and it is destroyed after a minute without hovering the tabs, so it does not keep a web view in memory.

- Interface: the `tabHoverCard` action (`src/lib/actions/tabHoverCard.ts`) handles the delay and closing, and sends requests through `src/lib/services/bridge/tab-hover-card.ts`. Each request has an increasing revision, so a late request can never reopen a closed card.
- Rust: `src-tauri/src/window/tab_hover_card/` keeps the card state and the pictures (`mod.rs`), takes pictures (`capture.rs`) and manages the window (`surface.rs`). Commands: `set_tab_hover_card` and `refresh_tab_previews`.
- Pictures: on Windows, WebView2 runs the DevTools method `Page.captureScreenshot` without a clip or scale, and Spark scales the image down. With a clip or scale, Chromium draws the page again at another size for the capture, which makes the page on screen flash. On Linux, WebKitGTK takes a snapshot of the visible region, which cairo scales down.
- The card page (`public/tab-hover-card.html`) writes the texts as text and shows only `data:image/` pictures.
- The card is not shown while a browser panel is open, during a tab drag or while Spark is closing.
- A failed picture is written to the error log as a Warning, with the tab it was for, and the card keeps the previous picture.
