# Optimization work

Changes are delivered in separate, validated commits. Native behavior must also be checked on the target operating systems; automated checks alone do not establish user-interface latency or memory savings.

## Geometry and IPC

Viewport measurements are coalesced by animation frame. Only one resize request is in flight, with the latest measurement retained and identical successful updates skipped. Geometry updates bypass tab operations and database locks. Full lifecycle reconciliation uses a map of WebView labels instead of repeated linear searches.

## Sessions and ordering

Session saves are transactional incremental upserts with prepared statements: unchanged rows are not rewritten, and only removed tabs are deleted. Dragging changes a local preview, commits once on release and restores the order on cancellation or failure. Reorder requests are serialized. History and downloads have chronological indexes; adaptive hibernation obtains frequent tab IDs in one grouped query and prunes on its periodic pass. Row decoding and activation-write failures are returned.

Native navigation callbacks enqueue history/session writes on one worker. History reads/clears and normal shutdown flush preceding writes. Explicit tab commands still persist before returning. No page URLs are printed by the worker on errors.


## Navigation and dialogs

Address drafts are preserved while editing and reset on an explicit tab change or Escape. Native loading and document-title events replace fixed spinner timers. Windows WebView2 observers supply source changes (including SPA URLs), history availability and navigation failures; other engines currently leave history availability unknown. The HTTPS indicator describes the scheme, not a certificate audit.

Tab snapshots carry monotonic revisions; older replies are ignored. Per-tab navigation patches have their own revision and preserve newer metadata across full snapshots. Generation tokens reject callbacks from replaced views. Dialogs share initial focus, focus containment, Tab wrapping, Escape and restoration, including stacked dialogs and shortcut recording.

## Search and preference storage

History searches all stored visits with a bounded page size, literal wildcard escaping and a stable `(visited_at, id)` cursor. The interface debounces requests and discards stale responses. SQLite LIKE provides its built-in case matching; this is not a Unicode full-text index. Bookmarks are grouped by folder once per change.

Command suggestions are computed only while open, ranked by title/URL relevance, normalized for accents and capped per category. Preference access uses an adapter that handles denied storage and quota errors, retains in-memory state and notifies the user without logging stored values.

## Remaining stages

- Resident WebView limits, download tasks and local security boundaries.
- Modularization, diagnostics, continuous validation and measured release guidance.
