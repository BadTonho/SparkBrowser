# Named sessions and Tools

Open **Tools → Sessions**, or use the Command Bar, to save the current workspace's pages. Sessions can be renamed, updated from the current workspace, deleted after confirmation and restored into the current workspace. Restoring adds new tab IDs without closing or reusing existing tabs. Only the selected eligible page is initially opened; other pages start hibernated.

Snapshots include titles, URLs, order, pins and selection. Forms, scroll and page navigation history are not captured. Shield-blocked pages remain hibernated and produce a visible warning. SQLite migrations and session updates are transactional; failed updates retain the previous saved content. This feature requires the desktop runtime. Data stays in the application's local database.

Tools shares the browser overlay lifecycle, keyboard focus containment, Escape and focus restoration. The Notes section provides [page notes](page-notes.md).
