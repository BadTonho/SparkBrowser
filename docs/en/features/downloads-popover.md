# Floating downloads panel and toolbar progress

Downloads opens 8 px below its toolbar button, aligned right and clamped to 8 px window margins. Width is 480 px, capped by available space; height follows content up to 560 px or the remaining window height. Long lists scroll internally. The card has an opaque theme background (`--bg-downloads-panel`); only the surrounding native surface remains transparent. Native pages and both split panes stay visible around the card without dimming, reloading or losing forms and scrolling.

Click the trigger again, click outside, press Escape or use Close to dismiss. The first outside click only closes the panel. Keyboard/header dismissal returns focus to the trigger. Another panel, application blur or a removed anchor closes Downloads without reclaiming focus. Shortcuts resolve Downloads or More in compact mode and restore the toolbar from Zen.

The URL form, expandable optional SHA-256 field, history, Pause, Resume, Restart, Cancel and verification remain available. The empty state requires both tasks and history to be empty.

## Progress border

Known-length transfers use `sum(clamped received bytes) / sum(total bytes)` across tasks currently Downloading. Paused and finished tasks are excluded. Changing the active set recalculates the percentage; it is not a permanent batch percentage. Any unknown/nonpositive total makes the border indeterminate.

With no transfers, Queued/Verifying/Finalizing tasks show processing without a percentage. When only Paused/Pausing tasks remain, a yellow static border retains known combined progress or freezes the segment for unknown totals. Completed downloads produce a full green border with a brief completion animation; Failed/VerificationFailed results produce a full static red border. Cancellation does not create an error notice. Priority is unseen error, transfer, processing, pause, unseen completion, then no indicator. Acknowledging an error returns the border to the remaining tasks' state.

Green and red notices persist across application restarts until Downloads is opened through the button, shortcut or menu entry. Clicking its trigger while the panel is open also acknowledges and closes it. New results while the panel is open create new notices; header Close, Escape and ordinary outside clicks do not acknowledge those results. More carries the same border in compact mode, with translated status in its Downloads entry; merely opening More does not acknowledge notices.

An initial snapshot and existing task events update the indicator while the panel is closed. Existing history on first adoption establishes a quiet baseline. A separate ledger in `safeStorage`, under `veloce_download_notices`, stores only task IDs, revisions, observed states and pending notices, never URLs, file names or paths. It suppresses old/repeated updates and persists acknowledgement; invalid data resets the baseline, and storage failure preserves session state through the existing storage warning. Removed tasks are pruned. This UI ledger is not included in backups and does not change transfer storage.

The SVG uses the theme accent during transfers and CSS variables `--download-status-completed`, `--download-status-paused` and `--download-status-error` for result states. It respects reduced motion and does not intercept pointers or change shortcut reordering. Translated tooltips and accessible descriptions identify the state without relying on color alone.

## Architecture and validation

Downloads and Tools share a safe presentation renderer/transport, with separate native services, documents, routes, tokens and sequence counters. Controllers and operations stay in the shell. The auxiliary document receives a presentation tree and sends typed actions against opaque control handles; it receives no application-command grants. Unknown/duplicate fields, invalid payloads/tokens, repeated sequences and closed sessions are rejected. Native progress elements are supported without executable markup or inline styles.

The web preview shares styles and geometry. Transfer storage, concurrency, resume rules and backups are unchanged. Run `npm run test:native -- --case downloads-popover` for the panel and `--case downloads` for transfer/restart/integrity checks. See [download behavior](downloads.md) and [native tests](../contributing/native-tests.md).
