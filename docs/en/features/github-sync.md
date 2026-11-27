# GitHub backup (private repository)

[Português](../../pt-BR/features/github-sync.md)

**Settings → Data & Backup → Sync** has a second card, **GitHub**, next to the sync card. It keeps an extra encrypted backup in a private repository of your GitHub account, **at the same time** as the sync in use (Google Drive, a folder, Nextcloud or WebDAV), with its own settings, state and errors. It uses a token you create yourself, so no app has to be registered.

## Setting it up

1. Turn on **backup encryption** (Automatic backups → Encryption). GitHub backups refuse to connect or send without it, and the card says so.
2. On GitHub, create a **private** repository with a README (an empty repository cannot receive files through the API).
3. Create a token in **Settings → Developer settings → Personal access tokens → Fine-grained tokens** with access to **this repository only** and the **Contents** permission set to **Read and write**.
4. Open the **GitHub** card, type the repository as `owner/name` (or paste its address), keep the branch `spark-sync` or type another one, paste the token and select **Connect**.

**Connect** checks everything on GitHub before saving: the token works, the repository is private, the token can write to it, the branch is not the default branch, and the branch holds only Spark's files. The first backup is sent right away.

The closed card shows the GitHub logo (in the text color, so it reads in both themes), **Account connected** while a token is saved, and the time of the last backup. Its border shows the same ring as the sync card: green while sending, yellow when automatic sending is off or the automatic sync is paused, red after an error.

To change the branch or token later, edit the fields and select **Save changes** (an empty token field keeps the saved token). **Disconnect** forgets the repository and token on this computer; the token stays valid on GitHub until you delete it there.

## How often it sends

A full backup goes to the repository **at most once an hour**, when something changed. The hour is counted from the last backup saved on disk (or the last attempt in this run, so a failure is not retried every minute), so reopening Spark does not send again. Closing Spark does not add a send. A pause of the automatic sync (Frequency and pause) pauses it too.

In the open card:

- **Send now** sends a backup at once, even if nothing changed.
- **Review** reads the backup in the repository and opens it in the review panel of Data & Backup: nothing changes until it is confirmed there.
- **Earlier versions** lists the copies kept on the branch and opens one in the same review panel.

The GitHub backup always holds every group (workspaces and tabs, bookmarks, sessions, notes, site search, settings, shortcuts): **What syncs** only applies to the sync target.

## What goes to the repository

Spark writes only to its branch (`spark-sync` by default) and never to the default branch:

- `browser.spark-backup.json`: the encrypted backup;
- `versions/browser.spark-backup.<unix seconds>.json`: the newest 5 earlier copies.

Each send replaces the branch with **one commit without history** (a force update) holding those files. The encrypted file changes completely on every send, so keeping each send as a commit would make the repository bigger every hour; with a single commit, GitHub drops the old copies on its own.

## Safety checks

- **Before every send**, Spark checks again that the repository is private, that the token can write and that the branch is not the default one. A repository made public later stops the backups with a clear message, and nothing is sent.
- A branch holding a file Spark did not write is never replaced (`backup.github_branch_in_use`). A protected branch is reported instead of forced.
- Without encryption nothing is sent, even if encryption is turned off after connecting.
- The token is kept in the system credential store (Windows Credential Manager or the Linux Secret Service) under its own key, apart from the sync target's password, and is never shown again. Fine-grained tokens expire; when one does, the card says the token was refused and asks for a new one.
- Requests go only to `https://api.github.com`, without following redirects.

## How it works

- Rust: `src-tauri/src/backup/github.rs` is the GitHub client: it checks the repository, branch and token, reads the branch (`git/ref`, `git/commits`, `git/trees`) and sends with the Git Data API (a blob for the file, a tree that reuses the blobs of the kept versions, a commit without parents and a forced update of the branch). `plan_tree` decides which files the commit holds and refuses a branch with other files.
- `src-tauri/src/backup/github_backup.rs` is the second destination: its settings, last send and change fingerprint are saved in the profile database under their own keys, its token in the credential store, and its last error in memory. It does not touch the sync target (`sync.rs`).
- Commands (`src-tauri/src/commands/backup_github.rs`): `get_github_backup_state`, `connect_github_backup` (checks on GitHub before saving), `set_github_backup_auto`, `disconnect_github_backup`, `push_github_backup`, `review_github_backup`, `list_github_backup_versions` and `review_github_backup_version`.
- Interface: `src/lib/stores/githubBackupStore.ts` loads the state when Spark starts and checks every minute whether an hourly backup is due (`githubDue`); reviews go through `reviewFrom` in `backupStore.ts`, the same review panel as the sync target. The card is in `BackupSettings.svelte`, beside the sync card; when there is room, it opens toward the left so it stays inside the category.
- Failures are written to the error log with the repository and the step (connect, send or download), and each has its own message in both languages (`backup.error_github_*`).

Tests: `src-tauri/src/backup/github.rs` (repository, branch and token checks, which files a send keeps, refusing a branch with other files, version order) and `tests/backup-github.test.mjs` (commands registered end to end, the card beside the sync target, the hourly rule, the safety checks, messages in both languages).
