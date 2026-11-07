# 📂 Estrutura de Diretórios do Projeto

> 🌍 **Idioma:** [English (Main)](../../en/architecture/project-structure.md) | **Português (pt-BR)**

O repositório segue uma separação modular rigorosa de responsabilidades tanto no **Frontend Svelte (`src/`)** quanto no **Core em Rust (`src-tauri/`)**.

---

## 1. Arquitetura do Frontend (`src/`)

```text
src/
├── App.svelte                         # Layout raiz, timer de auto-hibernação e atalhos globais
├── main.ts                            # Ponto de entrada da aplicação
├── styles/
│   └── theme.css                      # Variáveis de glassmorphism e temas dinâmicos em CSS
└── lib/
    ├── types/
    │   └── browser.ts                 # Interfaces TypeScript (TabItem, WorkspaceItem, DownloadItem, etc.)
    ├── services/
    │   └── tauri-bridge.ts            # Ponte IPC para navegação, abas, workspaces, downloads e SQLite
    ├── stores/
    │   ├── browserStore.ts            # Estado multi-abas, ações de hibernação e animação spring
    │   ├── commandBarStore.ts         # Estado da Command Bar flutuante e animação spring de escala
    │   ├── workspaceStore.ts          # Estado de Workspaces e injeção de temas dinâmicos CSS
    │   ├── downloadsStore.ts          # Estado do Gerenciador de Downloads e painel (Ctrl+J)
    │   ├── settingsStore.ts           # Modal de Configurações (Ctrl+,), Modo Zen (Ctrl+Shift+Z) e timer de RAM
    │   ├── persistenceStore.ts        # Estado reativo de Favoritos e Histórico em SQLite (Ctrl+H)
    │   └── privacyStore.ts            # Métricas do AdBlock Shield nativo e otimização de RAM
    └── components/
        ├── titlebar/
        │   └── Titlebar.svelte        # Barra de pesquisa/navegação, seletor horizontal de workspaces, ferramentas e controles da janela
        ├── bookmarks/
        │   └── BookmarkBar.svelte     # Barra de bookmarks com rolagem horizontal abaixo da pesquisa
        ├── sidebar/
        │   └── Sidebar.svelte         # Barra lateral do layout Vertical com workspaces, bookmarks e abas do workspace
        ├── workspaces/
        │   └── WorkspaceSwitcher.svelte # Seletor de workspaces na barra horizontal e na barra lateral vertical
        ├── tabs/
        │   ├── TabList.svelte         # Lista vertical animada de abas
        │   └── HorizontalTabBar.svelte # Barra horizontal de abas no topo
        ├── privacy/
        │   └── PrivacyShieldCard.svelte # Card do AdBlock Shield, contador de trackers e otimizador de RAM
        ├── history/
        │   └── HistoryDrawer.svelte   # Painel deslizante de Histórico de Navegação SQLite (Ctrl+H)
        ├── downloads/
        │   └── DownloadsDrawer.svelte # Painel deslizante do Gerenciador de Downloads (Ctrl+J)
        ├── settings/
        │   └── SettingsModal.svelte   # Modal de Configurações, Temas, Modo Zen e Atalhos (Ctrl+,)
        ├── commandbar/
        │   └── CommandBar.svelte      # Command Bar flutuante estilo Spotlight (Ctrl+T / Ctrl+K)
        └── viewport/
            └── ViewportFrame.svelte   # Quadro do site (ou tela cheia no Modo Zen) sincronizado com Rust
```

---

## 2. Arquitetura do Core em Rust (`src-tauri/`)

```text
src-tauri/
├── Cargo.toml                         # Dependências Rust (tauri v2, window-vibrancy, rusqlite bundled)
├── tauri.conf.json                    # Configuração de janela transparente e sem bordas
├── capabilities/
│   └── default.json                   # Permissões de segurança granulares do Tauri v2
└── src/
    ├── main.rs                        # Ponto de entrada do binário
    ├── lib.rs                         # Inicialização do Tauri, SQLite, restauração de sessão e comandos
    ├── database/
    │   ├── mod.rs
    │   ├── connection.rs              # Inicialização do SQLite no app_data_dir do SO (modo WAL)
    │   ├── history_repo.rs            # Operações CRUD do Histórico de Navegação
    │   ├── bookmarks_repo.rs          # Operações CRUD de Favoritos e favoritos iniciais
    │   ├── downloads_repo.rs          # Operações CRUD do Gerenciador de Downloads
    │   └── session_repo.rs            # Persistência automática de sessão por Workspace e restauração
    ├── privacy/
    │   ├── mod.rs
    │   └── adblock_engine.rs          # Motor nativo Rust de bloqueio de anúncios/trackers e script cosmético
    ├── state/
    │   ├── mod.rs
    │   ├── browser_state.rs           # Estado thread-safe de workspaces, abas, SQLite e AdBlock
    │   ├── tab_model.rs               # Estrutura TabInfo com workspace_id, hibernação e timestamps
    │   └── workspace_model.rs         # Definições de WorkspaceItem e paletas de cores dinâmicas
    ├── window/
    │   ├── mod.rs
    │   ├── vibrancy.rs                # Efeito nativo de vidro Mica / Acrylic no SO
    │   └── webview_manager.rs         # Ciclo de vida Multi-WebView, despertar sob demanda e injeção de shield
    └── commands/
        ├── mod.rs
        ├── navigation.rs              # Comandos IPC de navegação da aba ativa
        ├── tabs.rs                    # Comandos IPC de múltiplas abas, reordenação (reorder_tabs), movimentação entre workspaces e hibernação de RAM
        ├── workspaces.rs              # Comandos IPC de alternância de Workspaces
        ├── downloads.rs               # Comandos IPC do Gerenciador de Downloads
        ├── persistence.rs             # Comandos IPC de Histórico e Favoritos em SQLite
        └── privacy.rs                 # Comandos IPC do AdBlock Shield e métricas de privacidade
```

## 3. Ciclo de vida, downloads e testes de regressão

- `window/keyboard_bridge.rs`: integração de teclado da página e do sistema.
- `window/internal_overlays.rs`: WebViews dos overlays e transporte interno de mensagens.
- `window/url_input.rs`: resolução de URLs e pesquisas.
- `window/content_lifecycle.rs`: reconstrução das instâncias após mudanças de privacidade.
- `commands/overlays.rs`: visibilidade dos overlays, separada das operações de abas.
- `downloads/transfer.rs`: transferências HTTP, arquivos temporários e salvamento sem sobrescrita.
- `privacy/content_shield.js`: hooks de requisições com a lista de hosts compartilhada do Rust.
- `lib/stores/workspaceSelectionStore.ts`: seleção compartilhada sem ciclo de imports.
- `lib/components/status/BrowserErrorBanner.svelte`: exibição de erros do backend e tarefas assíncronas.
- `tests/`: testes JavaScript de regressão; testes Rust ficam junto aos módulos.
- `tools/vendor/deepmerge/`: dependência de build com licença e correção local.
- `updates/`, `commands/updates.rs`, `database/updates_repo.rs`: seleção nativa de releases, instalação assinada e preferências de canal.
- `tools/release/`: sincronização local de versões, preparação de assinatura e empacotamento; `src-tauri/tests/updater_signatures.rs` exercita o atualizador oficial com chaves isoladas.
