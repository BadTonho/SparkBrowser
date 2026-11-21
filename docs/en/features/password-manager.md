# Password manager

[Português](../../pt-BR/features/password-manager.md)

Open **Settings → Passwords** to keep your logins in an encrypted vault on this computer.

> **Not independently audited yet.** The vault is new and has not been reviewed by an independent security auditor. Settings shows this notice permanently. Do not rely on it as the only copy of important passwords. See the [security review](../security/account-containers-vault-review.md).

## What it does

- **Create a vault** with a master password (8 to 256 characters). Spark cannot recover this password for you.
- **Recovery key:** when the vault is created you get a recovery key, shown **once**. Spark does not keep a copy. It opens the vault if you forget the master password. Losing both the password and the recovery key makes the saved passwords unreadable.
- **Add, edit, search and delete** entries with a title, website, username, password and notes. A **Generate** button makes a random 20-character password (no look-alike characters such as `0`/`O` or `1`/`l`).
- **Copy a password or username** to the clipboard. The clipboard is emptied again after 30 seconds. Passwords are shown only when you ask (Show / Edit), never in the list.
- **Lock:** manually (**Lock now**), automatically after 1, 5 (default), 15, 30 or 60 minutes without use, and whenever the browser closes. A locked vault asks for the master password or the recovery key.
- **Change the master password** (the current one is asked again). Saved entries and the recovery key keep working.
- **Start over** if both secrets are lost: the unlock screen offers to erase the vault. This deletes every saved password for good.

This first version only manages the vault in Settings. It does not detect logins, offer to save passwords or fill them into pages, and it does not import or export passwords.

## How it is protected

- A random 256-bit key encrypts each entry (AES-256-GCM, a new random nonce each time, the entry id bound to the ciphertext). Titles, websites, usernames, passwords and notes are all encrypted; nothing readable is stored.
- That key is stored twice, each copy wrapped by a key derived with Argon2id (64 MiB, 3 passes): one from the master password, one from the recovery key. Changing the password only rewraps the key, so entries are not re-encrypted.
- The key lives in memory only while the vault is unlocked and is wiped when it locks. It is never written to disk.
- After 3 wrong guesses every further attempt has to wait, with the delay doubling up to 60 seconds.
- Only the browser's own interface can talk to the vault; web pages cannot reach it. **Private windows** refuse every vault command.
- The vault tables are **not** part of backups, WebDAV or Drive sync, or private-window profiles.
- Failures are written to the [error log](error-log.md) with what failed and where. The log never receives passwords, recovery keys, entry contents or keys.

## Limits

- Anyone who can run code as you while the vault is unlocked can read it. Lock the vault when you step away.
- Copying a password makes it available to every program that reads the clipboard until it is cleared.
- There is no unlock with the operating system account or biometrics, and no check of passwords against known leaks, yet.
