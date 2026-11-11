# Optimization work

Changes require local validation before release. Native behavior must also be checked on the target operating systems; automated checks alone do not establish user-interface latency or memory savings.

## Geometry and IPC

Viewport measurements are coalesced by animation frame. Only one resize request is in flight, with the latest measurement retained and identical successful updates skipped. Geometry updates bypass tab operations and database locks. Full lifecycle reconciliation uses a map of WebView labels instead of repeated linear searches.

## Sessions and ordering

Session saves are transactional incremental upserts with prepared statements: unchanged rows are not rewritten, and only removed tabs are deleted. Dragging changes a local preview, commits once on release and restores the order on cancellation or failure. Reorder requests are serialized. History and downloads have chronological indexes; adaptive hibernation obtains frequent tab IDs in one grouped query and prunes on its periodic pass. Row decoding and activation-write failures are returned.

Native navigation callbacks enqueue history/session writes on one worker. History reads/clears and normal shutdown flush preceding writes. Explicit tab commands still persist before returning. No page URLs are printed by the worker on errors.

Tab creation, selection, ordering, pinning, moves and resident-limit changes restore a checkpoint on persistence failure. Selection usage and session rows commit in the same transaction. Native documents are reconciled after persistence succeeds; a later display failure shows an error while preserving the committed session. Rollback preserves navigation/media observations received during the operation.

Bookmark, note, history and command-search database commands use a blocking executor with two worker permits and the existing connection mutex. Lock wait and SQL execution have separate diagnostic spans. The permit bounds active blocking tasks; it does not add connections or guarantee FIFO ordering for concurrent requests. Navigation writes retain their separate bounded worker and flushing rules.


## Navigation and dialogs

Address drafts are preserved while editing and reset on an explicit tab change or Escape. Native loading and document-title events replace fixed spinner timers. Windows WebView2 observers supply source changes (including SPA URLs), history availability and navigation failures; other engines currently leave history availability unknown. The HTTPS indicator describes the scheme, not a certificate audit.

Tab snapshots carry monotonic revisions; older replies are ignored. Per-tab navigation patches have their own revision and preserve newer metadata across full snapshots. Generation tokens reject callbacks from replaced views. Dialogs share initial focus, focus containment, Tab wrapping, Escape and restoration, including stacked dialogs and shortcut recording.

## Search and preference storage

History searches all stored visits with a bounded page size, literal wildcard escaping and a stable `(visited_at, id)` cursor. The interface debounces requests and discards stale responses. SQLite LIKE provides its built-in case matching; this is not a Unicode full-text index. Bookmarks are grouped by folder once per change.

Command suggestions are computed only while open, ranked by title/URL relevance, normalized for accents and capped per category. Preference access uses an adapter that handles denied storage and quota errors, retains in-memory state and notifies the user without logging stored values.

Persistent command search folds text on writes and maintains transactional SQLite FTS5 trigram indexes for notes, session names and session pages. Queries of at least three Unicode characters use literal trigram phrases; shorter queries scan the already folded fields. Search obtains three matches per category, returns two and reports whether more exist instead of calculating exact totals. Indexing costs additional storage and write work. Notes-panel search keeps its existing LIKE semantics and returns at most 100 summaries with 180-character previews; opening a note fetches its complete text.

The note writer evicts old saved drafts above 32 entries or 2,000,000 UTF-16 code units, preserving active editors and unsaved, saving or failed drafts. File-based backup imports keep the selected document in Rust behind an opaque token, release it on replacement/cancel/import and stream stale-preview fingerprints into SHA-256. Unchanged media reports no longer increment revisions or publish full snapshots; polling remains in place for video discovery.

## Resident memory and migrations

A configurable soft resident limit (default 8, range 1–100) hibernates the least recently used eligible tabs. Active, pinned, loading, edited-form, playing-media and downloading tabs are protected; the target can be exceeded when necessary. Pinned state survives session restoration. Automatic hibernation skips protected tabs and skips persistence/reconciliation when nothing changes. The resident cap still applies when the idle timer is off.

Activity detection is best effort for top-level DOM forms and audio/video; it cannot guarantee detection of iframe state, Web Audio, custom editors or application-specific unsaved work. Pin important pages explicitly. Only boolean activity flags are transported, with per-view generation checks.

Schema additions and chronological indexes use named transactional migrations. Existing palette `user_version` is imported into the shared migration ledger without reapplying a completed migration. Tests cover legacy session upgrades, idempotence, rollback and pin restoration.

## Downloads

Manual URL downloads run as revisioned tasks with progress, cancellation and a queue bounded to 64 pending tasks. Two shared semaphore slots cover manual and page-initiated transfers. HTTP clients are reused per shield policy, including validation of redirects. Cooperative cancellation checks each read; a blocked network operation can take up to its 15-second timeout to finish. Partial manual files are removed on cancellation/failure and existing files are never replaced.

Native page downloads retain the engine's authentication and blob support and use unique reserved destination names. When both slots are busy, page downloads are rejected with a visible retry message rather than replayed without cookies. The portable Tauri hook does not expose native progress/cancellation; those tasks show status only. Completed transfers are recorded separately in SQLite. Normal shutdown cancels manual tasks and allows up to 20 seconds for cleanup. Abrupt process termination cannot guarantee cleanup.

## Local security and live shield policy

Application commands are declared in the Tauri build manifest and explicitly granted only to the local `main` WebView. Window-wide grants and broad core defaults were removed. An additional command-handler label check rejects child WebViews even if they navigate to local resources. The local shell and overlays use CSP with self-hosted scripts; inline style attributes remain permitted for dynamic layout. This policy does not replace CSP supplied by remote websites.

Windows replaces its owned WebView2 document-created policy script and updates the live page wrappers, preserving forms, scroll position and native navigation history. Failed changes attempt to restore the previous policy. Other engines use the documented rebuild fallback, refuse changes while detected forms/media/downloads are protected and display the reset notice. Filtering remains best effort; existing network transfers are not retroactively undone.

## Module boundaries and startup

The bridge facade delegates to transport modules for navigation, overlays, workspaces, downloads, persistence, privacy, events and window controls. Workspace preview storage/migrations are separate from native IPC and validate restored profiles. Preview deletion receives the current tabs instead of inventing an initial session.

Settings categories, shortcut recording, the search catalog and shared scoped CSS are separate modules. Navigation page hooks, keyboard routing and content lifecycle reconciliation have separate Rust owners. Event listeners are registered before initialization; late registrations clean themselves up after unmount. The navigation writer has a bounded 256-message queue and coalesces session-only requests.

## Local and native validation

Checks run locally and in the pull-request/main CI workflow; the separate release workflow builds signed artifacts. Native WebDriver scenarios remain separate. See [native tests](../contributing/native-tests.md) for setup and scope. Native behavior must be validated on each target platform. Multi-monitor DPI, interactive latency and aggregate RAM comparisons remain measurement work; no savings are inferred from compilation or view counts.

## Idle and storage work

The permission popover follows its anchor only while needed and only publishes changed geometry. Default-volume pages do not schedule volume work for unrelated DOM changes, while non-default volume keeps observing dynamic players and the active-tab report loop remains in place. Diagnostic context collection no longer holds the tab-operation lock while awaiting the UI thread, and concurrent panel refreshes share a request.

Recent-site reads share a cache keyed by the new-tab revision, SQLite connection changes and `data_version`; history changes, icon updates and external writes cause recomputation. The same bounded result is used to revalidate card destinations. Windows extension synchronization uses the initialized connection, and archive/directory work runs in a bounded blocking executor. Hidden interface surfaces load independently on first use. Wallpaper records and the IndexedDB connection are reused; new images are stored as a single blob and older data URLs remain readable. Synthetic results and the native fast-build tradeoff are documented in [performance validation](../contributing/performance.md).

## Local diagnostics

Settings exposes bounded in-memory IPC timing samples, cumulative call/error counts, native operation spans, session rows written and resident/protected tab counts. Timing counters omit command arguments, page URLs, titles and tokens. The separate local `get_diagnostics.tab_details` inventory contains tab URLs/titles and workspace IDs; the response also contains the local application-data directory. Resource counters report counts of bridge tokens, shield scripts, media entries and Windows execution contexts without exposing their identifiers. Redact local inventories/paths before sharing a diagnostic response. See [performance validation](../contributing/performance.md) for repeatable comparisons and measurement limits.

## Startup, polling and storage

Cached filter lists are reused until they are 24 hours old. The background refresh waits for the remaining time instead of downloading and recompiling on every launch; a missing cache refreshes immediately. Engine compilation still runs synchronously during startup, so filter protection is active from the first request. Moving it off the main thread needs a serialized engine format and measurements first.

The panel host reads theme variables only after the root element's style, class or theme attributes change, instead of calling `getComputedStyle` on every frame. While a popover is open, anchor tracking still uses one frame callback, because CSS transforms can move the trigger without a resize.

Media polling continues for video discovery. It runs every 3 seconds when the focused tab has no audio, video or playing frames, and every second once media is present, so a video added to an idle page can take up to 3 seconds to appear.

SQLite connections set a busy timeout, in-memory temporary storage, an 8 MB page cache and a 256 MB memory-map limit. History and bookmark list queries use cached prepared statements.
