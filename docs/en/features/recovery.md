# Automatic session recovery

Spark records whether its previous execution ended cleanly. After an interruption it automatically restores every tab in the last committed SQLite session, preserving workspace, order, pin, selection and split layouts. A translated notice opens the tab organizer and can be dismissed. Recovery restores page addresses and metadata; forms, scroll position and internal page history are not saved.

Only the active tab or the current workspace's two split panes create WebViews at startup. Every other restored tab, including pinned tabs, starts hibernated. Returning to another workspace wakes its selected page or split pair normally.

An exclusive OS file lock (`profile.lock` in the application data directory) prevents simultaneous instances from changing the same profile's run marker. The OS releases the lock after a crash; the file is intentionally retained. The implementation uses Rust's [file locking API](https://doc.rust-lang.org/std/fs/struct.File.html#method.try_lock), available since Rust 1.89, without polling or deleting another process's lock file.

The named `session-recovery-run-state-v1` migration adds a run marker, not another copy of browsing data. Startup marks the run active, validates and loads the complete session, then confirms restoration in the same transaction as its tabs and split layouts. Application commands, navigation writes and page creation wait for startup. Read or write failures abort initialization without replacing the previous snapshot with a default or empty session.

Normal exit freezes and drains the navigation writer before acquiring the tab-operation lock. The final session, splits and clean marker commit together. Failures leave the run unclean for the next launch. The Windows updater follows the same rule before its installer can exit the process directly; if installer startup returns, the marker returns to running and navigation writes resume. Pending notes and page/download activity checks continue through the update preparation flow.

Run `npm run test:native -- --case recovery` to force-stop only an isolated test process and verify complete restoration, hibernation, split focus, the notice and a subsequent clean restart. `--case recovery-failure` injects a SQLite restoration failure in the stopped scenario's database, verifies the previous session remains complete and then successfully retries. Rust tests also exercise lock exclusivity, run states, rollback and shutdown persistence failure. Installer execution in a disposable VM remains outside this delivery.
