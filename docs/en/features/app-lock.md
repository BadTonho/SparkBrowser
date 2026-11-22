# App lock

[Português](../../pt-BR/features/app-lock.md)

The app lock asks for a secret every time a [profile](./profiles.md) opens, so nobody uses that profile's Spark without it. Each profile has its own lock, and it is off until you turn it on. Open **Settings → Profiles → App lock** to manage the lock of the profile you are using.

> **It only blocks access to Spark. It does not encrypt anything.** History, favorites and the other data of the profile stay readable to anyone who opens the data folder on this computer or copies it. Use it to keep other people from sitting down at your open computer and using your browser, not as protection against someone who can read your files. Spark says this in Settings and on the lock screen.

## Ways to open a locked profile

You can turn on one or more of these, and any of them opens the profile:

- **A password of its own** (8 to 256 characters), separate from every other password.
- **Windows Hello** (Windows only): PIN, fingerprint or face. It has its own key, separate from the password vault's.
- **The master password of the profile's password vault.** It only works when the vault has a master password. Spark checks it without opening the vault, so unlocking the app does not unlock the vault.

At least one way must stay on while the lock is on. If you rely only on Windows Hello, a Windows PIN reset or a new Windows account can lock you out, so Settings suggests adding a password too.

## Turning it on, changing it and turning it off

- **Turn on** by typing a new password and selecting **Lock with a password**, or by choosing Windows Hello or the vault master password as the first way.
- **Change or add a way** (a new password, Windows Hello, the vault master password) and **turn a way off** or **turn the lock off**: Spark asks you to confirm with a way that works now, even though the profile is already open. You cannot turn off the last way; turn the lock off instead.
- Turning the vault master password off, or erasing the vault, is refused while the lock opens **only** with that password: you would be locked out of your own browser. Add another way first.

## What the lock screen does

When a profile with a lock starts, Spark shows only the lock screen. **No tab, page or window of the profile exists behind it**, and no command of the browser works until the profile is open. A second launch of the same profile only brings the lock screen forward. Closing the window just ends the process, without saving anything.

- **Wrong secrets are slowed down.** After 3 wrong attempts every further attempt has to wait, the wait doubling up to a minute, even for the right secret. The wait is saved, so restarting Spark does not reset it.
- **Forgot the secret?** The lock encrypts nothing, so it cannot be removed without erasing what is behind it, or anyone could simply remove it. The lock screen offers **Erase the profile**, which asks for confirmation and then **permanently erases the history, favorites, cookies, logins, passwords and settings of that profile from this computer** and opens it again, empty and without a lock. Other profiles are not touched.

## Good to know

- The lock belongs to a profile and is **not** included in backups, WebDAV or Drive sync, and a private window does not carry it. A restored backup never brings a lock with it, and never removes one.
- The secrets are never stored: Spark keeps a hash of the password (stretched with the same Argon2id the password vault uses) and a hash of the signature Windows Hello produces.
- A profile that is open does not lock again by itself: the lock is asked when the profile starts.
- Every failure is written to the [error log](error-log.md) (area `app-lock`) without any secret.

## How it works (for developers)

- The lock is one row in the `app_lock` table of the profile's database: the verifiers, which ways are on and the brake on wrong guesses (failures and the time before which no attempt is accepted). `src-tauri/src/app_lock/` holds the logic (`ops.rs` works on a connection and takes the time as an argument; `verifier.rs` the hashes; `erase.rs` the helper that erases a profile).
- At start-up, once the database is open, a profile with a lock sets `app_locked` and stops before the last step. That step (`session_start::start`: the content view, the windows of the last session, the filter-list updates, the feeds and the moment the browser becomes ready) runs only when the profile has no lock or when the lock is opened. Because `startup_ready` stays false, every command except the few that unlock, quit or erase is refused (`commands::access::answers_before_ready`).
- Forgetting the secret starts a helper process (`--erase-profile <id>`) and ends this one. The helper waits for the profile lock, erases the data (never the other profiles, the profile picker's choice or the names of profiles) and starts the profile again.
- The brake (`src-tauri/src/brake.rs`) is shared with the password vault; only the app lock persists it.
