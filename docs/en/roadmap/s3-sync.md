# Sync with S3-compatible storage (future)

[Português](../../pt-BR/roadmap/s3-sync.md)

**Status:** planned, not started. It comes after [Nextcloud](../features/nextcloud-sync.md).

A sync mode for object storage that speaks the S3 API: Cloudflare R2, Backblaze B2, Wasabi, MinIO (self-hosted) and Amazon S3. One implementation serves all of them, and no app has to be registered anywhere: the person creates a bucket and an access key in their own provider.

## Why

- **Cheap or free:** the sync file is small, and R2 and B2 have free tiers far above it.
- **Self-hosting:** MinIO and similar servers run at home or on a VPS.
- **No account login in Spark:** only a key limited to one bucket, which the person can revoke.

It is meant for technical users: creating a bucket and a key in a provider's panel is the hard part.

## What the person fills in

- **Provider:** presets for R2, B2, Wasabi and AWS fill in the endpoint pattern; **Other** takes any endpoint.
- **Endpoint** (`https://`; `http://` only for `localhost`, like WebDAV) and **region** (`auto` for R2).
- **Bucket** and an optional **folder** (key prefix) inside it.
- **Access key ID** and **secret access key**.
- **Path-style addresses**, on for MinIO and most self-hosted servers.

## Plan

1. **New target** `SyncTargetConfig::S3 { endpoint_url, region, bucket, prefix, access_key_id, path_style, auto_save, apply_preferences_on_direct_pull }`. The secret key goes to the system credential store like a remembered WebDAV password (`backup/credentials.rs`), never into SQLite or the interface after it is saved.
2. **Signing:** AWS Signature Version 4 written in Rust (`backup/s3/sign.rs`, about 200 lines with the crypto crates Spark already ships), instead of the AWS SDK, which is large. Payloads are signed with their SHA-256, not `UNSIGNED-PAYLOAD`.
3. **Operations:** `PUT`, `GET`, `HEAD`, `DELETE`, `ListObjectsV2` and `CopyObject`, through the same blocking client as WebDAV (20-second timeout, no redirects).
4. **Versions:** the newest 5 copies under `<prefix>/versions/browser.spark-backup.<unix seconds>.json`, like a folder or WebDAV (`sync_versions.rs`). Bucket versioning is not used: not every provider has it, and it would make the list depend on the provider.
5. **Test connection:** a button that writes, reads and deletes a small test object, so a wrong key, region or permission shows up before the first sync.
6. **Interface:** a section in `BackupSettings.svelte` like WebDAV's, with the presets, and the same **Automatic**, **What syncs**, **Download when Spark opens** and **Earlier versions** as the other modes.

## Points to settle

- **Clock skew:** SigV4 rejects requests more than 15 minutes off. Show a clear message for `RequestTimeTooSkewed` instead of a generic authentication error.
- **Errors:** map `AccessDenied`, `NoSuchBucket`, `SignatureDoesNotMatch` and `InvalidAccessKeyId` to their own messages (`backup.s3_*`, in both languages).
- **Minimum permissions:** document the smallest policy that works (read, write, list and delete under the prefix only) for AWS, and the scoped tokens of R2 and B2.
- **B2:** use its S3-compatible endpoint, not its native API.

## Tests to add

- SigV4 against the published AWS test vectors.
- Address building in virtual-hosted and path style, with prefixes that need encoding.
- A full round trip (send, versions, receive) against a local test server, like the WebDAV test in `backup/tests.rs`.
