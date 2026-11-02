# Recently closed tabs

Use **Ctrl + Shift + T**, the Command Bar action or **Tools → Recently closed** to reopen a page. The shortcut can be changed in Settings. The desktop application keeps up to 100 closures for 30 days in its local SQLite database; browser preview displays the feature as unavailable.

Individual closures and reviewed organizer closures save the URL, title, workspace, position, pin and closure time in the same transaction as the session. Hibernation and workspace deletion do not add records. The organizer still revalidates protection before closing tabs. A failed transaction preserves the prior session and does not create or consume a closure record.

Reopening allocates a new tab ID, selects the original workspace and inserts the tab at the available original position. If that workspace no longer exists, the current workspace receives the page and the interface explains the fallback. The record is consumed only after persistence succeeds. Privacy Shield restrictions remain active; a denied reopening retains the record and shows an error.

The list can be cleared with confirmation. Clearing browsing history also clears recently closed tabs. Backups exclude these records. Reopening restores a page address and metadata; it does not restore forms, scroll position or internal navigation history.

Rust tests cover retention, workspace positions and transaction rollback. Run `npm run test:native -- --case closed-tabs` for individual/bulk closures, restart, new IDs, pin/position restoration, the real shortcut, workspace fallback, shield denial and history clearing.
