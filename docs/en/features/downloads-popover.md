# Floating downloads panel and toolbar progress

Downloads opens 8 px below its toolbar button, aligned right and clamped to 8 px window margins. Width is 480 px, capped by available space; height follows content up to 560 px or the remaining window height. Long lists scroll internally. The card has an opaque theme background (`--bg-downloads-panel`); only the surrounding native surface remains transparent. Native pages and both split panes stay visible around the card without dimming, reloading or losing forms and scrolling.

Click the trigger again, click outside, press Escape or use Close to dismiss. The first outside click only closes the panel. Keyboard/header dismissal returns focus to the trigger. Another panel, application blur or a removed anchor closes Downloads without reclaiming focus. Shortcuts resolve Downloads or More in compact mode and restore the toolbar from Zen.

The header reads **Downloads** and uses SVG buttons for the magnifying glass, Clear List (when finished entries exist), and Close. The link form starts collapsed on every opening. The magnifying glass toggles it and focuses the field. A successful submission clears the draft, collapses the form and returns focus to the magnifying glass; failure keeps the field and a short error visible. A blank submission still downloads the active page. Drafts survive toggling/closing during the session, but are never saved to disk. The empty state requires both tasks and history to be empty.

## Compact list and previews

Each row has a 40 × 40 px preview, a single-line filename with ellipsis and its full name in a tooltip, and a readable size below it. Active tasks show received/total bytes where known; completed tasks show their final size, and missing historical metadata says **Size unknown**. Pause, Resume, Cancel, Restart and the interrupted native download's source-page action use labeled SVG buttons. Restart still requires explicit confirmation. Small status symbols and accessible labels distinguish processing, pause, completion and failure. No file is automatically opened or executed.

URLs, local paths, checksum fields, hashes and technical errors are absent from this popover. Their future location remains undecided; checksum APIs, stored integrity information and existing verification behavior remain available internally.

On Windows, completed PNG/JPEG/GIF/BMP downloads can show a real local image preview, detected from file signatures rather than filename extensions. WIC uses explicit Microsoft decoders, the first frame, bounded dimensions and a PNG re-encode up to 96 × 96 px. It never uses Shell thumbnail handlers or launches files. Other formats/platforms and unavailable previews use local SVG file-type icons. Previews reject files over 16 MiB or 16 megapixels, corrupt images, directories, partials and links escaping the downloads directory. Only the validated, already-open file handle is read. Unknown formats require only a header read.

`get_download_visual` accepts a typed task/history identifier, never a path. Only the expected main shell document has permission. It resolves the registered completed file, returns optional size/PNG, and performs at most two blocking reads concurrently. Missing files fall back quietly. Its client cache lasts for one panel opening; pending replies cannot affect removed rows. A 512 KiB aggregate image budget keeps the shared native presentation bounded. The renderer accepts only bounded small PNG data URLs, never remote/local URLs or SVG images. Native and web form focus follow the same lifecycle. No download database or backup format changes are required.

## Progress border

Known-length transfers use `sum(clamped received bytes) / sum(total bytes)` across tasks currently Downloading. Paused and finished tasks are excluded. Changing the active set recalculates the percentage; it is not a permanent batch percentage. Any unknown/nonpositive total makes the border indeterminate.

With no transfers, Queued/Verifying/Finalizing tasks show processing without a percentage. When only Paused/Pausing tasks remain, a yellow static border retains known combined progress or freezes the segment for unknown totals. Completed downloads produce a full green border with a brief completion animation; Failed/VerificationFailed results produce a full static red border. Cancellation does not create an error notice. Priority is unseen error, transfer, processing, pause, unseen completion, then no indicator. Acknowledging an error returns the border to the remaining tasks' state.

Green and red notices persist across application restarts until Downloads is opened through the button, shortcut or menu entry. Clicking its trigger while the panel is open also acknowledges and closes it. New results while the panel is open create new notices; header Close, Escape and ordinary outside clicks do not acknowledge those results. More carries the same border in compact mode, with translated status in its Downloads entry; merely opening More does not acknowledge notices.

An initial snapshot and existing task events update the indicator while the panel is closed. Existing history on first adoption establishes a quiet baseline. A separate ledger in `safeStorage`, under `veloce_download_notices`, stores only task IDs, revisions, observed states and pending notices, never URLs, file names or paths. It suppresses old/repeated updates and persists acknowledgement; invalid data resets the baseline, and storage failure preserves session state through the existing storage warning. Removed tasks are pruned. This UI ledger is not included in backups and does not change transfer storage.

The SVG uses the theme accent during transfers and CSS variables `--download-status-completed`, `--download-status-paused` and `--download-status-error` for result states. It respects reduced motion and does not intercept pointers or change shortcut reordering. Translated tooltips and accessible descriptions identify the state without relying on color alone.

## Architecture and validation

Downloads and Tools share a safe presentation renderer/transport, with separate native services, documents, routes, tokens and sequence counters. Controllers and operations stay in the shell. The auxiliary document receives a presentation tree and sends typed actions against opaque control handles; it receives no application-command grants. Unknown/duplicate fields, invalid payloads/tokens, repeated sequences and closed sessions are rejected. Native progress elements are supported without executable markup or inline styles.

The web preview shares styles and geometry. Transfer storage, concurrency, resume rules and backups are unchanged. Run `npm run test:native -- --case downloads-popover` for the panel and `--case downloads` for transfer/restart/integrity checks. See [download behavior](downloads.md) and [native tests](../contributing/native-tests.md).
