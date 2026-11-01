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

## Resident memory and migrations

A configurable soft resident limit (default 8, range 1?100) hibernates the least recently used eligible tabs. Active, pinned, loading, edited-form, playing-media and downloading tabs are protected; the target can be exceeded when necessary. Pinned state survives session restoration. Automatic hibernation skips protected tabs and skips persistence/reconciliation when nothing changes. The resident cap still applies when the idle timer is off.

Activity detection is best effort for top-level DOM forms and audio/video; it cannot guarantee detection of iframe state, Web Audio, custom editors or application-specific unsaved work. Pin important pages explicitly. Only boolean activity flags are transported, with per-view generation checks.

Schema additions and chronological indexes use named transactional migrations. Existing palette `user_version` is imported into the shared migration ledger without reapplying a completed migration. Tests cover legacy session upgrades, idempotence, rollback and pin restoration.

## Downloads

Manual URL downloads run as revisioned tasks with progress, cancellation and a queue bounded to 64 pending tasks. Two shared semaphore slots cover manual and page-initiated transfers. HTTP clients are reused per shield policy, including validation of redirects. Cooperative cancellation checks each read; a blocked network operation can take up to its 15-second timeout to finish. Partial manual files are removed on cancellation/failure and existing files are never replaced.

Native page downloads retain the engine's authentication and blob support and use unique reserved destination names. When both slots are busy, page downloads are rejected with a visible retry message rather than replayed without cookies. The portable Tauri hook does not expose native progress/cancellation; those tasks show status only. Completed transfers are recorded separately in SQLite. Normal shutdown cancels manual tasks and allows up to 20 seconds for cleanup. Abrupt process termination cannot guarantee cleanup.

## Local security and live shield policy

Application commands are declared in the Tauri build manifest and explicitly granted only to the local `main` WebView. Window-wide grants and broad core defaults were removed. An additional command-handler label check rejects child WebViews even if they navigate to local resources. The local shell and overlays use CSP with self-hosted scripts; inline style attributes remain permitted for dynamic layout. This policy does not replace CSP supplied by remote websites.

Windows replaces its owned WebView2 document-created policy script and updates the live page wrappers, preserving forms, scroll position and native navigation history. Failed changes attempt to restore the previous policy. Other engines use the documented rebuild fallback, refuse changes while detected forms/media/downloads are protected and display the reset notice. Filtering remains best effort; existing network transfers are not retroactively undone.

## Remaining stages

- Modularization, diagnostics, continuous validation and measured release guidance.
