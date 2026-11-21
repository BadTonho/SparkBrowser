# Password manager

[Português](../../pt-BR/features/password-manager.md)

Open **Settings → Passwords** to keep your logins in an encrypted vault on this computer.

> **Not independently audited yet.** The vault is new and has not been reviewed by an independent security auditor. Settings shows this notice permanently. Do not rely on it as the only copy of important passwords. See the [security review](../security/account-containers-vault-review.md).

## What it does

- **Create a vault** and choose how it opens: a **master password** (8 to 256 characters), **Windows Hello** (Windows only) or both. Spark cannot recover a master password for you.
- **Ways to unlock are independent.** You can use only the master password, only Windows Hello, or both and pick one each time you unlock. Under **Vault options → Ways to unlock** you can turn each one on or off later (turning the master password off asks for it once more). At least one must stay on: the recovery key does not count as one.
- **Windows Hello** asks Windows to confirm it is you (PIN, fingerprint or face) every time. It only works on this computer and this Windows account. If a vault opens with Windows Hello only, a PIN reset, a new Windows account or a reinstall can lock you out of that method: use the recovery key.
- **Recovery key:** when the vault is created you get a recovery key, shown **once**. Spark does not keep a copy. It opens the vault if you forget the master password or Windows Hello stops working. Losing every way in, the recovery key included, makes the saved passwords unreadable.
- **Add, edit, search and delete** entries with a title, website, username, password and notes. A **Generate** button makes a random 20-character password (no look-alike characters such as `0`/`O` or `1`/`l`).
- **Copy a password or username** to the clipboard. The browser itself (not the Settings page) empties the clipboard again after 30 seconds, even if Settings was closed, but only if it still holds what was copied: something you copied afterwards is left alone. A password is copied straight from the encrypted vault, without passing through the interface. Passwords are shown only when you ask (Show / Edit), never in the list.
- **Lock:** manually (**Lock now**), automatically after 1, 5 (default), 15, 30 or 60 minutes without use, and whenever the browser closes. A locked vault asks for the master password, Windows Hello or the recovery key, depending on what is on.
- **Change the master password** (the current one is asked again). Saved entries, Windows Hello and the recovery key keep working.
- **Start over** if both secrets are lost: the unlock screen offers to erase the vault. This deletes every saved password for good.

This first version only manages the vault in Settings. It does not detect logins, offer to save passwords or fill them into pages, and it does not import or export passwords.

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
