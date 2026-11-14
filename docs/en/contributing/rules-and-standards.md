# 🛡️ Project Rules & Engineering Standards

> 🌍 **Language:** **English** | [Português (pt-BR)](../../pt-BR/contributing/rules-and-standards.md)

To maintain a high-quality, secure, and scalable open-source project, all contributors and automated agents must follow these seven core rules:

---

## Rule 1: Zero Sensitive Data in Version Control
- **Never upload or commit sensitive files** to GitHub (including `.env`, API keys, private keys, certificates, credentials, or local `.db`/`.sqlite` user files).
- **Mandatory End-of-Process Audit:** At the end of every development task and prior to any commit, verify `git status`, `git diff`, and `.gitignore` to ensure zero sensitive data exposure.

## Rule 2: Impeccable Project Organization
- Keep the directory tree clean, modular, and intuitive at all times.
- Create dedicated folders and split files whenever necessary so each module has a single, clear responsibility.

## Rule 3: Modular & Well-Structured Documentation
- Never let documentation become a single cluttered file.
- Organize documentation into logical subdirectories (`architecture/`, `features/`, `roadmap/`, `contributing/`) and keep every topic concise and up to date.

## Rule 4: Bilingual Documentation (English Primary + Portuguese Translation)
- **English (`docs/en/` and root `README.md`)** is the primary documentation language.
- **Portuguese (`docs/pt-BR/`)** must always be maintained in parallel, mirroring the exact folder and file structure of `docs/en/`.

## Rule 5: Strict User Authorization (No Unprompted Actions)
- **Never start coding, modifying files, or taking actions without an explicit request from the user.**
- When the user asks a question, clarify or answer directly. Never preemptively edit codebase files unless explicitly requested.

## Rule 6: Dual Platform Support (Windows & Linux)
- The application must support and run seamlessly on both **Windows** and **Linux**.
- Architecture choices and implementations must preserve cross-platform compatibility across both operating systems (using conditional compilation `#[cfg(...)]` or platform abstractions where appropriate).
- **Testing Flexibility:** Executing automated/native tests on both operating systems simultaneously is **not mandatory** for each development run or environment.

## Rule 7: Changelog for Every Release
- Version **0.1.0** is the initial release and has **no changelog**. Every later version must have one.
- Changelogs live in the root `changelog/` folder, **one subfolder per version** named exactly like the version (for example `changelog/0.2.0/`). Each subfolder contains one file per language: `en.md` (English) and `pt-BR.md` (Portuguese), kept in sync.
- Every changelog is split into three categories, in this order: **New features**, **Improvements** and **Fixes** (`Novas funcionalidades`, `Melhorias` and `Correções` in Portuguese). Omit a category that has no entries.
- **Keep the changelog up to date as you work:** any change a user can notice (new behavior, better behavior or a bug fix) must add an entry to the changelog of the version in progress, in both languages, in the same change. Changelogs of already released versions are never rewritten.
- The format, writing guidelines and an example are in the [changelog guide](./changelog.md).
