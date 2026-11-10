# State ownership and operation contracts

`BrowserState` is a facade over private session, policy, surface, security and infrastructure storage. Existing IPC names and serialized payloads remain unchanged. No database or backup migration is required.

## Session and document observations

Organization commands live under `state/session_ops`; `commands/` keeps compatibility facades. Only the state module can acquire mutable session storage. Other features receive read-only query guards. The session operation guard serializes organization changes, including backup and startup handoffs.

Use `TabCheckpoint` for reversible changes: capture metadata, apply the change, reconcile visibility and residency, persist, then touch native documents and emit the committed snapshot. Its transaction extension can include another domain's database writes; a failure rolls back both the database and in-memory organization. Account deletion markers and session changes use this boundary. Never hold a synchronous state guard across an await or native callback.

Native navigation, media and downloads report through `observe_document`. Its input exposes URL, title, navigation and activity only. It cannot move a tab, change an account, select a workspace or recreate a removed tab. The service increments the observation revision only when something changes. Import handoffs preserve observations received while the database operation was running.

Policies expose read-only guards or value queries and explicit setters. Surface and security services retain scoped synchronization ports for native window handlers; these ports do not grant mutable access to session organization. Repository connections remain owned by infrastructure and must follow the existing database executor and transaction rules.

## Native content lifecycle

The content lifecycle owns creation, reconciliation and cleanup; its private factory constructs platform WebViews. Existing `webview_manager` imports remain valid through the facade. Reconciliation reuses existing documents. Closing and resetting use the same cleanup path for permission requests, native downloads, media sessions, internal-page bridges and keyboard tokens.

Each resident document records its resolved profile. Reconciliation compares that binding with committed metadata, including hidden tabs after workspace removal. It recreates documents only when necessary; moving a container tab between workspaces preserves the document because its container profile stays the same.

On Windows, the first navigation waits until native handlers have been installed on the blank WebView. Requests arriving during installation replace the pending destination, so delayed startup cannot overwrite the user's URL. Completion is bound to the WebView generation; callbacks from closed or recreated documents cannot dispatch an old destination.

Passive activity and internal new-tab messages use the existing native execution-context registry on Windows so they cannot cancel a navigation in progress. The receiver checks the live context, current origin, generation, bounded envelope, allowed channel and existing authenticated query parameters. Messages wait for the document binding to become available. Linux retains the navigation transport. Neither path grants application IPC permissions to sites.

Persist profile changes before closing the affected document. Geometry-only changes use viewport operations and must not rebuild pages. Native failures after commit must be reported as display failures; do not pretend that a committed database transaction was rolled back.

## Frontend ownership

The browser facade delegates to session actions, private confirmed state, address editing and presentation modules. Public tabs, confirmed URL, workspace selection, workspace list and split layout stores expose subscriptions, not setters. Consumers use actions; the architecture check restricts private mutation imports.

Drag order is a presentation overlay over confirmed tabs. Canceling or failing a drag removes the overlay, so a recently closed tab cannot reappear. A failed workspace move leaves the most recent confirmed session intact. Snapshot revisions reject old command replies; newer document observations survive an older organizational snapshot.

The browser lifecycle registers listeners before bootstrap reads and ignores responses after disposal. Shared domain lifecycles retain one subscription/timer set per JavaScript realm, with reference-counted, idempotent cleanup. Late subscription registrations are immediately released. Workspace bootstrap cannot overwrite a selection received from a newer event. Media, downloads, permissions, backup, zoom, PiP, updates, shutdown, drag, workspace shortcuts and internal-page preferences use this ownership mechanism.

Media, zoom and privacy panels coordinate mutual exclusion through a small surface coordinator. Their stores do not import one another. A delayed opening loses ownership when a newer panel is requested.

Projected panels discard focus work from older presentations and ignore closure of an older session after reopening. Downloads retain a pending focus request when submission collapses the form before its button is enabled; ordinary progress updates do not steal focus.

See [regression workflow](../contributing/regression-workflow.md) for automated enforcement and required scenarios.
