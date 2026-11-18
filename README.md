<div align="center">

# Spark

**A fast, private desktop browser built with Rust, Tauri and Svelte.**

Native web views, workspaces, split view and local-first privacy tools, in one customizable interface.

[![CI](https://github.com/BadTonho/SparkBrowser/actions/workflows/ci.yml/badge.svg)](https://github.com/BadTonho/SparkBrowser/actions/workflows/ci.yml)
[![License: Apache-2.0](https://img.shields.io/badge/license-Apache--2.0-blue.svg)](./LICENSE)
![Platforms](https://img.shields.io/badge/platforms-Windows%20%7C%20Linux-lightgrey)
![Rust](https://img.shields.io/badge/Rust-2021-orange?logo=rust&logoColor=white)
![Tauri](https://img.shields.io/badge/Tauri-2-24C8DB?logo=tauri&logoColor=white)
![Svelte](https://img.shields.io/badge/Svelte-5-FF3E00?logo=svelte&logoColor=white)

[Documentation](./docs/en/README.md) · [Português (Brasil)](./docs/pt-BR/README.md) · [Features](./docs/en/features/core-features.md) · [Architecture](./docs/en/architecture/overview.md) · [Contributing](#contributing)

</div>

---

> [!NOTE]
> Spark is in **early development** (latest release 0.1.2, version 0.1.3 in progress). Expect rough edges and changes between releases. Windows is the most tested platform; Linux support is described in the [Linux support guide](./docs/en/features/linux-support.md).

## Highlights

### Browsing and workspaces

- **Native web views:** each tab keeps its page and navigation state while you switch between tabs. Spark uses the web view runtime of your operating system, so there is no bundled browser engine.
- **Workspaces and layouts:** group tabs into workspaces, arrange them horizontally or vertically, or use [split view](./docs/en/features/split-view.md) and Zen mode.
- **Command bar:** search, switch tabs and run actions from the [command bar](./docs/en/features/command-search.md), with [keyboard shortcuts](./docs/en/features/keyboard-shortcuts.md) across tabs.
- **Tab care:** [hibernation](./docs/en/features/hibernation.md) for idle tabs, restore recently closed tabs, and [tab media controls](./docs/en/features/tab-media.md).

### Privacy and security

- **Ad and tracker shield:** a network rule engine with bundled filter lists, plus [link cleaning](./docs/en/features/tracking-protection.md) of tracking parameters.
- **Ephemeral tabs:** [disposable tabs](./docs/en/features/ephemeral-tabs.md) with isolated, temporary storage that is deleted when the tab closes.
- **Workspace isolation (opt-in):** [separate cookies and storage per workspace](./docs/en/features/workspace-isolation.md), off by default.
- **You decide:** per-site [permissions](./docs/en/features/site-permissions.md), local history and privacy settings.

### Tools and customization

- **New tab page:** an offline-first page with recent sites and optional [clock, weather, Gmail and wallpaper widgets](./docs/en/features/new-tab-widgets.md).
- **Reader view and dark mode:** a distraction-free [reader](./docs/en/features/reader-mode.md) and an optional [dark mode for websites](./docs/en/features/forced-dark-mode.md).
- **Built-in tools:** [downloads manager](./docs/en/features/downloads.md), [feed reader](./docs/en/features/feed-reader.md), notes, bookmarks with [export](./docs/en/features/bookmark-export.md), and several [search engines](./docs/en/features/search-engines.md).
- **Local backup:** export and review your settings, workspaces and bookmarks in a [versioned file](./docs/en/features/backup.md).
- **Extensions (Windows):** [Chrome WebExtensions support](./docs/en/features/extensions.md), depending on the web view runtime.
- **Updates:** [stable and beta channels](./docs/en/features/updates.md) with signed releases.

See the [feature guide](./docs/en/features/core-features.md) for details and platform notes.

## Platform support

| | Windows | Linux |
| --- | :---: | :---: |
| Web engine | WebView2 | WebKitGTK |
| Core browsing, workspaces, split view | ✅ | ✅ |
| Extensions | ✅ | Not yet |
| Release packages | Installer (x64) with signed updates | `.deb`, `.rpm`, AppImage, `.tar.gz` |

Behavior can differ between platforms because each uses its own system web view. Feature pages call out the differences where they matter.

## Get started

### Prerequisites

- [Node.js and npm](https://nodejs.org/)
- [Rust](https://www.rust-lang.org/tools/install) with its platform build tools
- The native web view dependencies required by [Tauri 2](https://v2.tauri.app/start/prerequisites/)

### Run in development

```bash
npm install
npm run tauri dev
```

On Windows PowerShell, use `npm.cmd` if the execution policy prevents `npm` from running:

```powershell
npm.cmd install
npm.cmd run tauri dev
```

## Project stack

| Area | Technology |
| --- | --- |
| Desktop shell and native features | Rust, Tauri 2, Wry |
| Browser interface | Svelte 5, TypeScript, Vite |
| Local persistence | SQLite |
| Web content | The operating system's web view runtime |

## Contributing

Bug reports, usability feedback and code contributions are welcome. Before opening a change, read the [project standards](./docs/en/contributing/rules-and-standards.md) and run the checks below. The [project structure guide](./docs/en/architecture/project-structure.md) helps you find the right area to update.

```bash
npm run check      # Svelte and TypeScript checks
npm test           # unit and documentation tests
npm run validate   # broader workflow: frontend and Rust checks
```

See the [validation guide](./docs/en/contributing/validation.md) for platform notes and optional native tests.

npm installs [local push checks](./docs/en/contributing/local-hooks.md) outside CI. Ordinary pushes run `npm run validate:quick`; release-tag pushes require full validation and fresh critical native tests. Existing checkouts can run `npm run hooks:install`. GitHub CI remains manual.

Changes that users can notice also need an entry in the [changelog](./changelog/README.md), following the [changelog guide](./docs/en/contributing/changelog.md).

## Documentation

The documentation is maintained in English and Brazilian Portuguese.

- [Documentation home](./docs/en/README.md) · [Início da documentação](./docs/pt-BR/README.md)
- [Feature guides](./docs/en/features) · [Guias de funcionalidades](./docs/pt-BR/features)
- [Architecture overview](./docs/en/architecture/overview.md) · [Project structure](./docs/en/architecture/project-structure.md)
- [Security boundaries](./docs/en/security/application-boundaries.md) · [Validation and contribution notes](./docs/en/contributing/validation.md)

## Security

To report a vulnerability, follow the [security policy](./.github/SECURITY.md). Please do not open a public issue for security problems.

## Acknowledgements

Spark stands on the shoulders of excellent open-source projects and services, including [Tauri](https://tauri.app/), [Svelte](https://svelte.dev/), [adblock-rust](https://github.com/brave/adblock-rust), [EasyList](https://easylist.to/), [Open-Meteo](https://open-meteo.com/) and [Simple Icons](https://github.com/simple-icons/simple-icons). See the full list in [Acknowledgements and third-party licenses](./docs/en/legal/acknowledgements.md).

## License

Spark is licensed under the [Apache License 2.0](./LICENSE). The bundled EasyList, EasyPrivacy and NoCoin filter lists are third-party data under their own licenses; see [their notice](./src-tauri/resources/filterlists/README.md).
