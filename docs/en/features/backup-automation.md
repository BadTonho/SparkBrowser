# Automatic backups, history and encryption

Settings → Data & Backup includes an **Automatic backups** panel built on top of the reviewed import described in [Local backup and reviewed import](backup.md). Desktop builds only.

## Scheduled backups

- **Back up automatically** keeps a rolling history in the application data directory (`backups/`). Frequency is every 6 hours, daily or weekly.
- **Also back up when the browser closes** adds a snapshot during shutdown. A failure never delays or blocks closing the browser.
- **Back up now** creates a snapshot on demand and always writes one.
- The interface checks the schedule once a minute and shortly after startup, so a backup that came due while the browser was closed is taken on the next launch. Scheduled and on-close snapshots compare the streamed SHA-256 fingerprint of data and preferences with the previous snapshot and skip identical content, only refreshing the schedule clock. After a failure the next attempt is delayed by 15 minutes.
- Snapshots are captured through the same flush, session lock and consistency rules as manual export.

## History and restore

- Each snapshot is named `backup-<epoch>-<manual|scheduled|exit>.veloce-backup.json`. The panel lists date, origin, size and whether it is encrypted, newest first.
- **Versions to keep** (1–100, default 10) prunes the oldest snapshots after every backup. Lowering the value prunes immediately.
- **Restore…** loads the snapshot into the existing review flow: counts, additions, skipped conflicts, optional preferences and stale-preview protection all apply. Nothing changes until **Confirm import**. Restoring is still a merge: current values win on conflicts.
- **Delete** needs a second click on the same button to confirm.
- Snapshot identifiers are validated against the generated name pattern, so a file name from the interface can never reach outside the backup directory. Foreign files and temporary exports in that directory are ignored.

## Password protection

- **Protect backups with a password** encrypts every backup this application writes: manual exports, history snapshots and the sync target (Google Drive, synced folder and WebDAV). The file keeps the `.veloce-backup.json` extension.
- Format `veloce-backup-encrypted` (envelope version 1): the key is derived with Argon2id (64 MiB, 3 iterations, 1 lane, random 16-byte salt) and the document is sealed with AES-256-GCM using a random 12-byte nonce. Cost parameters are stored in the envelope and bounded when reading, so a hostile file cannot demand unbounded memory or time. A wrong password and a modified file are indistinguishable by design.
- The password is stored only in the system credential store (Windows Credential Manager, or Linux Secret Service through `secret-tool`), under a key separate from the WebDAV secret. It never enters SQLite, backups, logs or the private-profile copy. Passwords have 8 to 256 characters.
- If encryption is on but the password cannot be read, writing fails with an explicit error instead of producing an unencrypted file.
- **Choose backup…**, **Restore…** and sync pulls try the stored password first. When it is missing or does not match (for example an older backup created with a previous password), the interface asks for the password; a wrong attempt keeps the file selected so the user can retry, and **Cancel** releases it. Plaintext backups remain readable with or without a password.
- Changing the password only affects new backups: older ones keep the password they were created with. **Losing the password makes encrypted backups unrecoverable.**
- Encrypted files may be about a third larger than the 32 MiB plaintext limit because of base64; the plaintext limit itself is unchanged.

## Verification

Rust tests cover envelope roundtrip, wrong password, tampering, hostile KDF parameters, fresh salt and nonce per file, encrypted file and sync roundtrips, stored-password behavior, schedule validation and persistence, snapshot naming, traversal rejection, ordering, uniqueness within one second and retention. `npm test` checks command ACL coverage and mirrored catalogs.
