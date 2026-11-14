# Project Rules & Engineering Standards (`AGENTS.md`)

These rules are mandatory for all development, file operations, documentation, and version control tasks in this repository.

---

## 1. Security & Sensitive Files (Zero-Leak Policy)
- **Never commit or upload sensitive files** to Git/GitHub (e.g., `.env`, `.env.*`, API keys, tokens, private keys `*.pem`/`*.key`/`*.p12`/`*.pfx`, credentials, local user databases `*.db`/`*.sqlite`, signing certificates, or build secrets).
- **Mandatory Final Verification:** At the end of every workflow, feature implementation, or before any Git commit/push operation, explicitly verify (`git status`, `git diff`, and `.gitignore` check) that no sensitive data, hardcoded secrets, or temporary credentials are staged or exposed.

## 2. Impeccable Project Organization
- Always maintain a clean, modular, and scalable folder and file structure.
- Proactively create dedicated directories, subdirectories, and separate modules whenever a file grows too large or mixes concerns (e.g., separating UI components, stores, Rust commands, database handlers, and window management).
- Keep the project root clean; only essential configuration files belong at the root level.

## 3. Modular & Impeccable Documentation
- Documentation must always be kept up-to-date, well-formatted, and thoroughly organized alongside code changes.
- **Split files and use subfolders:** Never dump all documentation into a single monolithic file. Organize documentation by domain into clear subdirectories (e.g., `architecture/`, `features/`, `roadmap/`, `contributing/`, `security/`).

## 4. Bilingual Documentation Standard (EN Primary + PT-BR Translation)
- **Primary Language:** All primary documentation (including the root `README.md` and `docs/en/`) must be written in **English**.
- **Portuguese Translation:** Every documentation file in `docs/en/` must have a corresponding, synchronized **Portuguese (`pt-BR`)** translation under `docs/pt-BR/` mirroring the exact same folder and file structure.

## 5. Strict User Authorization (No Unprompted Actions)
- **Never start coding, modifying files, or executing actions without explicit user request.**
- If the user asks a question, seeks clarification, or discusses an idea, **only answer the question or discuss**. Never preemptively edit code or execute changes unless the user explicitly tells you to proceed or make the change.

## 6. Dual Platform Support (Windows & Linux)
- The application must support and run seamlessly on both **Windows** and **Linux**.
- Architecture choices and implementations must preserve cross-platform compatibility across both operating systems (using conditional compilation `#[cfg(...)]` or platform abstractions where appropriate).
- **Testing Flexibility:** Executing automated/native tests on both operating systems simultaneously is **not mandatory** for each development run or environment.

## 7. Changelog for Every Release
- Version **0.1.0** is the initial release and has **no changelog**. Every later version must have one.
- Changelogs live in the root `changelog/` folder, **one subfolder per version** named exactly like the version (for example `changelog/0.2.0/`). Each subfolder contains one file per language: `en.md` (English) and `pt-BR.md` (Portuguese), kept in sync.
- Every changelog is split into three categories, in this order: **New features**, **Improvements** and **Fixes** (`Novas funcionalidades`, `Melhorias` and `Correções` in Portuguese). Omit a category that has no entries.
- **Keep the changelog up to date as you work:** any change a user can notice (new behavior, better behavior or a bug fix) must add an entry to the changelog of the version in progress, in both languages, in the same change. Changelogs of already released versions are never rewritten.
- The format, writing guidelines and an example are in [`docs/en/contributing/changelog.md`](docs/en/contributing/changelog.md).
