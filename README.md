# 🌐 Open-Source Modern Web Browser (Rust + Svelte)

> 🌍 **Language / Idioma:** **English (Main)** | [Português (pt-BR)](./docs/pt-BR/README.md)

A next-generation, open-source web browser built with **Rust (Tauri v2 / Wry)** and **Svelte (TypeScript + Modern CSS)**. Designed from the ground up for **extreme speed, minimal RAM usage**, and **fluid 60/120 FPS animations** with native OS glassmorphism.

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

- **Ultra-Lightweight Core (Rust):** Powered by `Tauri v2` and `wry` Multi-WebView, resulting in a ~3–5 MB binary and instant startup.
- **Spring-Physics UI (Svelte):** Compiled without a Virtual DOM, utilizing `svelte/motion` and modern CSS for effortless 60/120 FPS animations.
- **Native Glassmorphism:** Hardware-accelerated **Mica / Acrylic** window effects via `window-vibrancy` in Rust.
- **Privacy & Speed by Default:** Native Rust ad and tracker blocker (`adblock-rust`) and automatic inactive tab hibernation to save RAM.
