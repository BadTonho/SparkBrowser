# 📖 Documentation Index (English)

> 🌍 **Language:** **English** | [Português (pt-BR)](../pt-BR/README.md)

Welcome to the official documentation of the Open-Source Browser project. All documentation is modularly organized into thematic subdirectories.

---

## 📂 Sections

### 1. 🏗️ Architecture & Technology Stack
- [Architecture Overview](./architecture/overview.md): Hybrid **Rust (Tauri v2 / Wry)** + **Svelte** architecture, Multi-WebView window composition, and native GPU vibrancy.
- [State ownership and operation contracts](./architecture/state-ownership.md): Domain boundaries, committed sessions and native document lifecycle.
- [Project Directory Structure](./architecture/project-structure.md): Modular breakdown of `src/` (Svelte) and `src-tauri/` (Rust).

### 2. [✨ Core Features & Visual Experience](./features/core-features.md)
Specifications for the floating Command Bar (`Ctrl + T`), smart tab hibernation, Privacy Shield, Workspaces, and Zen Mode.

### 3. [🗺️ Development Roadmap](./roadmap/phases.md)
Step-by-step milestones from Phase 1 (MVP Prototype) to Phase 4 (Advanced Polish & Customization).

### 4. [🛡️ Project Rules & Engineering Standards](./contributing/rules-and-standards.md)
Mandatory repository guidelines covering security (zero-leak policy), folder organization, and bilingual documentation maintenance.

- [Validation](./contributing/validation.md)
- [Regression prevention workflow](./contributing/regression-workflow.md)
- [Local native integration](./contributing/native-tests.md)
- [Performance validation](./contributing/performance.md)
- [Optimization work](./architecture/optimization-work.md)
- [Named sessions and Tools](./features/named-sessions.md)
- [Site search shortcuts](./features/site-search.md)
- [Search engines](./features/search-engines.md)
- [Address bar suggestions](./features/address-suggestions.md)
- [Color palettes](./features/color-palettes.md)
- [Password manager](./features/password-manager.md)
- [Profiles](./features/profiles.md)
- [App lock](./features/app-lock.md)
- [Autofill (cards and contact info)](./features/autofill.md)
- [Command Bar appearance](./features/command-bar.md)
- [Resources panel](./features/resources.md)
- [Upgrading from the Veloce name](./features/rename-migration.md)
- [Hardware acceleration](./features/hardware-acceleration.md)
- [Dark mode for websites](./features/forced-dark-mode.md)
- [Page notes](./features/page-notes.md)
- [Feed reader](./features/feed-reader.md)
- [Tab organizer](./features/tab-organizer.md)
- [Split view](./features/split-view.md)
- [Linux support](./features/linux-support.md)
- [Site permissions](./features/site-permissions.md)
- [Dependency notes](./security/dependency-notes.md)
- [Security policy and reporting vulnerabilities](./security/reporting-vulnerabilities.md)
- [Acknowledgements and third-party licenses](./legal/acknowledgements.md)
- [Release channels and updates](./features/updates.md)
- [Local signed releases](./contributing/releases.md)
- [Changelog guide](./contributing/changelog.md)
- [Interface languages](./features/localization.md)
- [Local backup and reviewed import](./features/backup.md)
- [Automatic backups, history and encryption](./features/backup-automation.md)
- [Selective restore and replace mode](./features/selective-restore.md)
- [Backup integrity and versions](./features/backup-integrity.md)
- [Bookmark export for other browsers](./features/bookmark-export.md)
- [Recently closed tabs](./features/closed-tabs.md)
- [Automatic session recovery](./features/recovery.md)
- [Resumable downloads and checksums](./features/downloads.md)
- [Floating downloads panel and toolbar progress](./features/downloads-popover.md)
- [Protected shutdown](features/shutdown.md)
- [Several browser windows](features/windows.md)
- [Tab diagnostics](features/tab-diagnostics.md)
- [Persistent page zoom](features/site-zoom.md)
- [Custom new tab page](features/new-tab.md)
- [Notes and sessions in the Command Bar](features/command-search.md)
- [Keyboard shortcuts across tabs](features/keyboard-shortcuts.md)
- [Tab media controls](features/tab-media.md)
- [Protection pop-up and Auto skip ads](features/auto-skip-ads.md)
- [Correctness fixes and verification — November 2, 2026](reviews/2026-11-02/correctness-fixes.md)
- [Reorder quick actions](./features/titlebar-actions.md)
- [Floating Tools panel](features/tools-popover.md)

- [Chrome WebExtensions support](features/extensions.md)
- [Immersive Reader View](features/reader-mode.md)
- [Ephemeral Tabs and disposable containers](features/ephemeral-tabs.md)
- [Tab menu, dividers and groups](features/tab-groups.md)
- [Application security boundaries](security/application-boundaries.md)
- [Network privacy: HTTPS-only mode and DNS over HTTPS](features/network-privacy.md)
- [Network privacy investigation](security/network-privacy-investigation.md)
- [Tracking protections](features/tracking-protection.md)
- [Workspace content isolation](features/workspace-isolation.md)
- [AppContainer isolation (planned)](security/app-container-isolation.md)
- [Planned protection against suspicious sites and downloads](roadmap/security-protections.md)
