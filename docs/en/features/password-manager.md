# Password manager

[Português](../../pt-BR/features/password-manager.md)

Open **Settings → Autofill & Passwords → Password manager** to keep (the same section also holds [cards and contact info](./autofill.md))  your logins in an encrypted vault on this computer. Once unlocked it has two pages, like the password manager of other browsers: **Passwords** (search, **Add** and the list) and **Settings** (everything about the vault itself).

> **Not independently audited yet.** The vault is new and has not been reviewed by an independent security auditor. Settings shows this notice permanently. Do not rely on it as the only copy of important passwords. See the [security review](../security/account-containers-vault-review.md).

## What it does

- **Create a vault** and choose how it opens: a **master password** (8 to 256 characters), **Windows Hello** (Windows only) or both. Spark cannot recover a master password for you.
- **Ways to unlock are independent.** You can use only the master password, only Windows Hello, or both and pick one each time you unlock. Under **Vault options → Ways to unlock** you can turn each one on or off later (turning the master password off asks for it once more). At least one must stay on: the recovery key does not count as one.
- **Windows Hello** asks Windows to confirm it is you (PIN, fingerprint or face) every time. It only works on this computer and this Windows account. If a vault opens with Windows Hello only, a PIN reset, a new Windows account or a reinstall can lock you out of that method: use the recovery key.
- **Recovery key:** when the vault is created you get a recovery key, shown **once**. Spark does not keep a copy. It opens the vault if you forget the master password or Windows Hello stops working. Losing every way in, the recovery key included, makes the saved passwords unreadable. Next to **Copy key** there is **Download key**, which saves the key to a text file you choose (it only accepts text shaped like a recovery key). The file is **not encrypted**: keep it away from this computer, for example on a USB drive.
- **The list is grouped by site.** Every account of one site sits under one row (with a letter badge and, for several accounts, "N accounts"). Click a row to open it (a site with several accounts first lists them). Searching opens every matching site.
- **Seeing a password asks who you are, every time.** Clicking a login asks you to confirm with the **master password** or **Windows Hello** (only Windows Hello, if "Use Windows Hello when showing or copying passwords" is on). The backend checks the proof, not the interface: a wrong master password counts toward the brake on wrong guesses. Then the login opens in its own page: the username and the password (hidden behind dots until you press the eye) each with a copy button, the website, the note, and **Edit** and **Delete**.
- **Find a login** with the search field next to **Add password**. Sites with several accounts show a count beside the name. Select the site to see its accounts, then select the login you want to open.
- **Away for 5 minutes, it closes.** While a password is on screen (or being edited), five minutes without using the window, with the tab out of sight, or with the computer asleep or locked, close it and bring you back to the list. You have to confirm again to see it. This is separate from the vault's own automatic lock (1 to 60 minutes), which locks the whole vault.
- **Add, edit, search and delete** entries with a title, website, username, password and notes. A **Generate** button makes a random 20-character password (no look-alike characters such as `0`/`O` or `1`/`l`).
- **Copy a password or username** to the clipboard. The browser itself (not the Settings page) empties the clipboard again after 30 seconds, even if Settings was closed, but only if it still holds what was copied: something you copied afterwards is left alone. A password is copied straight from the encrypted vault, without passing through the interface. Passwords are never in the list: they appear only in a login's page, after you confirm who you are, and are copied from that page.
- **Lock:** manually (**Lock now**), automatically after 1, 5 (default), 15, 30 or 60 minutes without use, and whenever the browser closes. In the unlock card, enter your master password and press Enter or select **Unlock**. When both methods are configured, **Unlock with Windows Hello** appears below **Unlock**. If you only use Windows Hello, it is the main action. To use the recovery key, select **Use the recovery key instead** at the bottom of the card.
- **Change the master password** (the current one is asked again). Saved entries, Windows Hello and the recovery key keep working.
- **Start over** if both secrets are lost: the unlock screen offers to erase the vault. This deletes every saved password for good.

- **Use Windows Hello when showing or copying passwords** (Settings page, needs Windows Hello as a way to unlock): Windows confirms it is you every time a password is shown, copied or exported. Turning this off asks for Windows Hello once more, and turning Windows Hello off as a way to unlock also turns this off.
- **Import passwords** from a CSV file (the format Chrome, Edge and most managers export: `name, url, username, password, note`, with other common column names accepted too). Spark opens the file dialog and reads the file itself, so its contents never pass through the interface. Passwords already in the vault (same address, user name and password) are not added twice, and rows without a password or with fields that are too long are left out. The result is shown: added, already there, left out. The CSV file stays on disk: delete it afterwards.
- **Export passwords** to a CSV file. Spark first confirms it is you (Windows Hello when the vault has it, otherwise your master password), then asks where to save. **The file is not encrypted**: anyone who opens it can read every password, so delete it as soon as you are done.
- **Delete all data** erases the vault and the recovery key (after a confirmation), also while it is unlocked.

Logins are [filled into pages](./autofill.md#filling-in-pages) when you click a login field. Spark also [offers to save or update a login](./autofill.md#saving-what-you-type) when you submit a form.

## How it is protected

- A random 256-bit key encrypts each entry (AES-256-GCM, a new random nonce each time, the entry id bound to the ciphertext). Titles, websites, usernames, passwords and notes are all encrypted; nothing readable is stored.
- That key is stored once per way of unlocking, each copy wrapped by its own key. The master password and the recovery key each go through Argon2id (64 MiB, 3 passes). Windows Hello works differently: Windows keeps a key pair for the vault and signs a fixed message only after you confirm with PIN, fingerprint or face. That signature, which only that key pair can produce, wraps the third copy. Spark stores nothing secret for it. Changing or turning off a way of unlocking only rewraps or removes that copy, so entries are not re-encrypted.
- A failed Windows Hello unlock is not counted as a wrong guess (Windows already controls it), but a wrong master password or recovery key still is.
- The key lives in memory only while the vault is unlocked and is wiped when it locks. It is never written to disk.
- After 3 wrong guesses every further attempt has to wait, with the delay doubling up to 60 seconds.
- Only the browser's own interface can talk to the vault; web pages cannot reach it. **Private windows** refuse every vault command.
- The vault tables are **not** part of backups, WebDAV or Drive sync, or private-window profiles.
- Failures are written to the [error log](error-log.md) with what failed and where. The log never receives passwords, recovery keys, entry contents or keys.

## Limits

- Anyone who can run code as you while the vault is unlocked can read it. Lock the vault when you step away.
- Copying a password makes it available to every program that reads the clipboard until it is cleared. On Windows the copy is marked so it stays out of the clipboard history and cloud clipboard, and it is also cleared if the browser exits first. On Linux the copy disappears when the browser exits, but a clipboard manager may already have kept its own copy.
- Windows Hello is only offered on Windows. Linux has no equivalent yet: there the vault opens with the master password. The Windows Hello step has not been tested on every kind of device (PIN only, fingerprint reader, camera).
- Windows Hello proves that someone who can sign in to Windows is present. It is not a second secret: anyone who can run code as you while the vault is unlocked can still read it.
- There is no check of passwords against known leaks yet.
