# Resumable local downloads

The manager opens below its toolbar button and shows combined progress on the button border; see [the floating panel](downloads-popover.md).

The download manager supports Pause, Resume, Restart, Cancel and optional SHA-256 verification. URL transfers and their partial files persist locally. Restarting the application restores their state without opening connections; resume is always explicit. Up to two operations run simultaneously, with at most 64 unfinished tasks including paused transfers.

URL transfers use asynchronous HTTP, a 20-second connection timeout and a 30-second read inactivity timeout, without an overall duration limit while data progresses. Resumption requires a strong ETag, the same final URL and a matching `206 Content-Range`. Weak/missing validators, changed representations, ignored ranges and inconsistent responses require an explicit restart. Current shield and redirect restrictions apply to each new request.

Partials live in `.veloce-partials` under the selected OS Downloads directory. Durable checkpoints store the confirmed byte count and SHA-256 of that prefix. Before resuming, Veloce rehashes the prefix, rejects damaged/missing data and truncates any unconfirmed tail. Completed files publish without overwriting existing paths. The finalization journal recovers interruptions between publishing the file and recording completion; the task ID prevents duplicate history records. Partials remain until completion or explicit cancellation/restart. Clearing history does not delete files or paused tasks.

An optional expected SHA-256 is checked before a URL transfer is marked complete. Completed downloads can also be checked again against an expected hash. Verification runs outside the UI thread; a mismatch never deletes a completed file. Hashes are integrity checks, not malware scanning.

Windows page downloads use WebView2 operations directly for progress, pause, conditional resume and cancellation. COM objects stay on the UI thread and commands validate WebView generation. Paused native transfers keep their originating WebView protected. An engine operation cannot be recovered after restarting the application; interrupted native tasks provide their source page rather than replaying authenticated, POST or blob requests through the URL downloader. Completed native files remain verifiable. Linux/macOS retain the engine download flow without advertising these additional native controls.

New commands and events are local-shell only. Transfer journals use the named `resumable-downloads-v1` migration, remain outside backups and never sync to a server. Tests redirect Downloads only for the isolated native-test identifier, inside the scenario's data directory.

Run `npm run test:native -- --case downloads` to exercise restart, byte-for-byte equality, invalid ranges/validators, local corruption, checksums and native Windows controls. Rust tests cover range parsing, journal rollback, idempotent completion, path validation and collision-safe publishing.
