# 🌐 Open-Source Modern Web Browser (Rust + Svelte)

> 🌍 **Language / Idioma:** **English (Main)** | [Português (pt-BR)](./docs/pt-BR/README.md)

An open-source desktop web browser built with **Rust (Tauri v2 / Wry)** and **Svelte (TypeScript)**, with persistent tab WebViews, local SQLite storage, and native window effects.

---

## 🚀 Quick Start (Phase 1 MVP)

```bash
# Install frontend dependencies
npm install

# Run the browser in development mode (Rust + Svelte + Multi-WebView)
npm run tauri dev
```

---

## 📚 Documentation Index

Our documentation is organized into modular categories and available in both **English** (primary) and **Portuguese (`pt-BR`)**:

| Category | English (Primary) | Português (pt-BR) |
| :--- | :--- | :--- |
| **📖 Documentation Home** | [docs/en/README.md](./docs/en/README.md) | [docs/pt-BR/README.md](./docs/pt-BR/README.md) |
| **🏗️ Architecture & Stack** | [Architecture Overview](./docs/en/architecture/overview.md) | [Visão Geral da Arquitetura](./docs/pt-BR/architecture/overview.md) |
| **📂 Project Structure** | [Directory Structure](./docs/en/architecture/project-structure.md) | [Estrutura de Diretórios](./docs/pt-BR/architecture/project-structure.md) |
| **✨ Core Features** | [Core Features & UX](./docs/en/features/core-features.md) | [Funcionalidades Principais](./docs/pt-BR/features/core-features.md) |
| **🗺️ Roadmap** | [Development Phases](./docs/en/roadmap/phases.md) | [Fases de Desenvolvimento](./docs/pt-BR/roadmap/phases.md) |
| **🛡️ Rules & Standards** | [Project Rules & Security](./docs/en/contributing/rules-and-standards.md) | [Regras e Padrões do Projeto](./docs/pt-BR/contributing/rules-and-standards.md) |

---

## ⚡ Key Highlights

- **Rust Core:** Powered by `Tauri v2` and `wry` Multi-WebView, using the operating system WebView runtime. Binary size and memory usage depend on the platform and build profile.
- **Spring-Physics UI (Svelte):** Compiled without a Virtual DOM, utilizing `svelte/motion` and modern CSS for effortless 60/120 FPS animations.
- **Native Glassmorphism:** Hardware-accelerated **Mica / Acrylic** window effects via `window-vibrancy` in Rust.
- **Privacy & Speed by Default:** Known-host navigation filtering, best-effort page request filtering, and real inactive-tab hibernation. See [privacy behavior](./docs/en/features/core-features.md) for coverage and reload behavior.

On Windows PowerShell, use `npm.cmd` if the execution policy blocks `npm.ps1`. See [validation](./docs/en/contributing/validation.md) and [dependency notes](./docs/en/security/dependency-notes.md) for checks and platform limitations.
