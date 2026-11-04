# Floating downloads panel and toolbar progress

Downloads opens 8 px below its toolbar button, aligned right and clamped to 8 px window margins. Width is 480 px, capped by available space; height follows content up to 560 px or the remaining window height. Long lists scroll internally. Native pages and both split panes stay visible without dimming, reloading or losing forms and scrolling.

Click the trigger again, click outside, press Escape or use Close to dismiss. The first outside click only closes the panel. Keyboard/header dismissal returns focus to the trigger. Another panel, application blur or a removed anchor closes Downloads without reclaiming focus. Shortcuts resolve Downloads or More in compact mode and restore the toolbar from Zen.

The URL form, expandable optional SHA-256 field, history, Pause, Resume, Restart, Cancel and verification remain available. The empty state requires both tasks and history to be empty.

## Progress border

Known-length transfers use `sum(clamped received bytes) / sum(total bytes)` across tasks currently Downloading. Paused and finished tasks are excluded. Changing the active set recalculates the percentage; it is not a permanent batch percentage. Any unknown/nonpositive total makes the border indeterminate.

With no transfers, Queued/Verifying/Finalizing tasks show processing without a percentage. When only Paused/Pausing tasks remain, a dim static border shows known combined progress or a fixed segment for unknown totals. Other terminal/interrupted states remain in the manager without keeping the border active.

More carries the same border in compact mode, with translated status in its Downloads entry. An initial snapshot and existing task events update it while the panel is closed. The SVG uses theme color, respects reduced motion and does not intercept pointers or change shortcut reordering.

## Architecture and validation

Downloads and Tools share a safe presentation renderer/transport, with separate native services, documents, routes, tokens and sequence counters. Controllers and operations stay in the shell. The auxiliary document receives a presentation tree and sends typed actions against opaque control handles; it receives no application-command grants. Unknown/duplicate fields, invalid payloads/tokens, repeated sequences and closed sessions are rejected. Native progress elements are supported without executable markup or inline styles.

The web preview shares styles and geometry. Transfer storage, concurrency, resume rules and backups are unchanged. Run `npm run test:native -- --case downloads-popover` for the panel and `--case downloads` for transfer/restart/integrity checks. See [download behavior](downloads.md) and [native tests](../contributing/native-tests.md).
