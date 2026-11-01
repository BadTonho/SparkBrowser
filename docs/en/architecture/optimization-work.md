# Optimization work

Changes are delivered in separate, validated commits. Native behavior must also be checked on the target operating systems; automated checks alone do not establish user-interface latency or memory savings.

## Geometry and IPC

Viewport measurements are coalesced by animation frame. Only one resize request is in flight, with the latest measurement retained and identical successful updates skipped. Geometry updates bypass tab operations and database locks. Full lifecycle reconciliation uses a map of WebView labels instead of repeated linear searches.

## Sessions and ordering

Session saves are transactional incremental upserts with prepared statements: unchanged rows are not rewritten, and only removed tabs are deleted. Dragging changes a local preview, commits once on release and restores the order on cancellation or failure. Reorder requests are serialized. History and downloads have chronological indexes; adaptive hibernation obtains frequent tab IDs in one grouped query and prunes on its periodic pass. Row decoding and activation-write failures are returned.

Native navigation callbacks enqueue history/session writes on one worker. History reads/clears and normal shutdown flush preceding writes. Explicit tab commands still persist before returning. No page URLs are printed by the worker on errors.

## Remaining stages

- Address drafts, navigation metadata and modal accessibility.
- History search, command ranking and reliable storage.
- Resident WebView limits, download tasks and local security boundaries.
- Modularization, diagnostics, continuous validation and measured release guidance.
