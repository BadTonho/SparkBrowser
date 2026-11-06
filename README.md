# Veloce Browser

**A desktop browser built with Rust, Tauri, and Svelte.** Veloce combines native web views with a customizable browser interface and local tools for managing tabs, workspaces, and browsing data.

[Documentation](./docs/en/README.md) · [Português (Brasil)](./docs/pt-BR/README.md) · [Architecture](./docs/en/architecture/overview.md) · [Contributing](./docs/en/contributing/validation.md)

## Highlights

- **Native web views:** each open tab keeps its page and navigation state while you switch between tabs. The system web view is used on each platform.
- **Flexible workspace:** arrange tabs horizontally or vertically, group them into workspaces, or use split view and Zen mode.
- **Personalized new tab:** configure the clock, weather, background media, and other widgets.
- **Useful browser tools:** search and switch tabs from the command bar, save bookmarks, manage downloads, and restore recently closed tabs.
- **Local controls:** manage browsing history, site permissions, privacy settings, and tab hibernation from the browser interface.

See the [feature guide](./docs/en/features/core-features.md) for details and behavior notes.

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

## Checks

Run the project checks before proposing a change:

```bash
npm run check
npm test
npm run validate
```

`npm run validate` runs the broader validation workflow, including frontend and Rust checks. See the [validation guide](./docs/en/contributing/validation.md) for platform notes and optional native tests.

## Documentation

The documentation is maintained in English and Brazilian Portuguese.

- [Documentation home](./docs/en/README.md) · [Início da documentação](./docs/pt-BR/README.md)
- [Feature guides](./docs/en/features) · [Guias de funcionalidades](./docs/pt-BR/features)
- [Architecture](./docs/en/architecture/overview.md) · [Architecture and project structure](./docs/en/architecture/project-structure.md)
- [Security boundaries](./docs/en/security/application-boundaries.md) · [Validation and contribution notes](./docs/en/contributing/validation.md)

## Contributing

Bug reports, usability feedback, and code contributions are welcome. Before opening a change, review the [project standards](./docs/en/contributing/rules-and-standards.md) and run the checks above. The [project structure guide](./docs/en/architecture/project-structure.md) can help you find the right area to update.

## Platform notes

Veloce uses the web view runtime provided by the operating system, so rendering behavior and some native capabilities can differ between Windows and Linux. Feature documentation calls out platform-specific behavior where it matters.
