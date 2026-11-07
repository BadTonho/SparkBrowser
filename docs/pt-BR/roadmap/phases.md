# 🗺️ Roadmap de Desenvolvimento

> 🌍 **Idioma:** [English (Main)](../../en/roadmap/phases.md) | **Português (pt-BR)**

## Fase 1: Protótipo Base (MVP) — ✅ Concluída
- [x] Inicializar estrutura do projeto **Tauri v2 (Rust) + Svelte + TypeScript**.
- [x] Configurar janela sem bordas padrão (*custom titlebar*) com efeito translúcido (`window-vibrancy`).
- [x] Implementar gerenciamento básico de **Multi-WebView** no Rust (interface do navegador + visualização do site).
- [x] Adicionar controles básicos de navegação: Barra de URL/Pesquisa, Voltar, Avançar e Recarregar.

## Fase 2: Sistema Multi-Abas e Animações Modernas — ✅ Concluída
- [x] Construir ciclo de vida de múltiplas abas no Rust (criar, alternar, fechar, rastrear título/URL).
- [x] Implementar animações com física de mola (`svelte/motion`) na troca de abas e barra lateral retrátil.
- [x] Criar a **Command Bar (`Ctrl + T` / `Ctrl + K`)** flutuante e animada.

## Fase 3: Performance, Privacidade e Persistência — ✅ Concluída
- [x] Integrar `rusqlite` para Histórico, Favoritos e Restauração de Sessão.
- [x] Adicionar verificacoes de hosts na navegacao e filtragem de fetch/XHR/beacon (`privacy/`).
- [x] Implementar o sistema automático e manual de **Hibernação de Abas** para otimização de RAM.

## Fase 4: Polimento e Recursos Avançados — ✅ Concluída
- [x] Adicionar **Workspaces** (*Personal*, *Work*, *Study*, *Gaming*) e temas dinâmicos customizáveis.
- [x] Construir um **Gerenciador de Downloads (`Ctrl + J`)** moderno e animado persistido em SQLite.
- [x] Adicionar **Modo Foco / Zen (`Ctrl + Shift + Z`)**, referência de atalhos de teclado e **Painel de Configurações (`Ctrl + ,`)**.
