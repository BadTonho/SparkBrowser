# Protected shutdown

Closing the window, Alt+F4 and normal exit requests keep the shell alive while pending notes are saved. A translated review lists edited forms, media and transfers across all workspaces. **Continue browsing** receives focus first. Cancelling a review leaves transfers running. A changed activity review requires another confirmation.

After **Exit anyway**, URL transfers pause and persist their checkpoints. Native page downloads lose their WebView connection; the review warns that they cannot resume after reopening. The navigation writer stops accepting work and drains before the final tab lock. Session tabs, split layouts and the clean-run marker commit together. Persistence failures keep the browser open and drafts available for retry.

The review lists each page with its host and a badge for what is at risk (edited form, page download, playing media). **Don't ask again when I close the browser** on that dialog, or **Skip the close confirmation** in Settings > Performance, stores a local preference (`spark_skip_exit_review`). It only skips closing the browser, and only when nothing would be lost for good: media and transfers that can resume. Edited forms and page downloads that cannot resume still show the review, and installing an update or restarting the engine always show it. The preference is saved when **Exit anyway** is chosen, and can be turned off again in Settings.

The updater uses the same coordinator and persistence sequence. Active transfers must finish or be cancelled before installation. If the installer returns, the run marker and navigation writer return to their running state. Preview cannot close a desktop process.

Run `node tools/testing/native-smoke.mjs --case shutdown` against the isolated test build for note debounce, save failure/retry, safe default focus, activity changes and clean restart. No installer is executed by this scenario.
