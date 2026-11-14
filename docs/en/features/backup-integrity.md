# Backup integrity and versions

## Integrity checksum

Backups written by this version use format version 5, which stores a SHA-256 `checksum` at the top level of the file. It covers the whole document (format, version, creation time, data and preferences) in canonical form: keys sorted and whitespace ignored, so reformatting the file with another tool does not invalidate it.

When a version 5 file is opened, from a file, the history, or a sync target, the checksum is verified before anything is parsed further:

- **Mismatch** (`backup.checksum_mismatch`): the file is damaged or was edited after saving. Nothing is imported.
- **Missing checksum** in a version 5 file (`backup.checksum_missing`): treated the same way, since removing it would otherwise skip the check.
- Encrypted backups are verified after decryption, in addition to the authentication AES-GCM already provides.

Files from versions 1 to 4 have no checksum and are still accepted without one. The checksum detects accidental corruption and hand edits; it is not a signature, so someone who can rewrite the file can also recompute it. Use encryption when tampering matters.

## Versions and upgrades

The file `version` is the schema version. Supported versions are 1 to 5.

- A file with a **higher version** than the app understands is rejected with `backup.newer_version` ("created by a newer version, update the app"), before strict parsing, instead of a generic "invalid file" error.
- Older files are validated with the rules of their own version, then upgraded to the current data shape by `backup/migrate.rs`, one step per version crossed. The document keeps its original `version`, which the review panel shows.

| Step | What it covers |
| --- | --- |
| 1 → 2 | Optional site zoom preferences; absent means none |
| 2 → 3 | `pip_window_shadow`; absent means `false` |
| 3 → 4 | Split layouts get explicit left/right tab groups; legacy splits put the other tabs on the left |
| 4 → 5 | File-level checksum only |

Steps that only add optional fields rely on deserialization defaults and need no code. When the format changes again, bump `CURRENT_VERSION`, accept the new version in validation and add a step to `migrate::upgrade`.

## Implementation

- `files::encode` computes the checksum; `files::decode_with` verifies it through `verify_integrity`.
- `BackupService::select` and structured previews call `migrate::upgrade`.
- Tests: `src-tauri/src/backup/integrity_tests.rs`.
