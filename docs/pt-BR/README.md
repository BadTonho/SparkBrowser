# 📖 Índice da Documentação (Português)

> 🌍 **Idioma:** [English (Main)](../en/README.md) | **Português (pt-BR)**

Bem-vindo à documentação oficial em Português do projeto de Navegador Open-Source. Toda a documentação é dividida em subpastas temáticas para manter a organização impecável.

---

## 📂 Seções

### 1. 🏗️ Arquitetura e Stack Tecnológica
- [Visão Geral da Arquitetura](./architecture/overview.md): Explicação detalhada da arquitetura híbrida **Rust (Tauri v2 / Wry)** + **Svelte**, composição de janela Multi-WebView e efeitos de vidro nativos na GPU.
- [Responsabilidade sobre estados e contratos de operações](./architecture/state-ownership.md): Fronteiras dos domínios, sessões confirmadas e ciclo dos documentos nativos.
- [Estrutura de Diretórios do Projeto](./architecture/project-structure.md): Organização modular de `src/` (Svelte) e `src-tauri/` (Rust).

### 2. [✨ Funcionalidades Principais e Experiência Visual](./features/core-features.md)
Especificações da Command Bar flutuante (`Ctrl + T`), hibernação inteligente de abas, Privacy Shield, Workspaces e Modo Zen.

### 3. [🗺️ Roadmap de Desenvolvimento](./roadmap/phases.md)
Etapas passo a passo desde a Fase 1 (Protótipo Base / MVP) até a Fase 4 (Polimento e Recursos Avançados).

### 4. [🛡️ Regras e Padrões do Projeto](./contributing/rules-and-standards.md)
Diretrizes obrigatórias do repositório cobrindo segurança (política de vazamento zero), organização de pastas e manutenção da documentação bilíngue.

- [Validação](./contributing/validation.md)
- [Fluxo de prevenção de regressões](./contributing/regression-workflow.md)
- [Integração nativa local](./contributing/native-tests.md)
- [Validação de desempenho](./contributing/performance.md)
- [Otimizações implementadas](./architecture/optimization-work.md)
- [Sessões nomeadas e Ferramentas](./features/named-sessions.md)
- [Atalhos de busca por site](./features/site-search.md)
- [Mecanismos de busca](./features/search-engines.md)
- [Atualizando a partir do nome Veloce](./features/rename-migration.md)
- [Aceleração de hardware](./features/hardware-acceleration.md)
- [Modo escuro nos sites](./features/forced-dark-mode.md)
- [Notas por página](./features/page-notes.md)
- [Leitor de feeds](./features/feed-reader.md)
- [Organizador de abas](./features/tab-organizer.md)
- [Visualização dividida](./features/split-view.md)
- [Suporte ao Linux](./features/linux-support.md)
- [Permissões por site](./features/site-permissions.md)
- [Dependencias](./security/dependency-notes.md)
- [Política de segurança e relato de vulnerabilidades](./security/reporting-vulnerabilities.md)
- [Créditos e licenças de terceiros](./legal/acknowledgements.md)
- [Canais de versão e atualizações](./features/updates.md)
- [Releases assinadas localmente](./contributing/releases.md)
- [Idiomas da interface](./features/localization.md)
- [Backup local e importação com revisão](./features/backup.md)
- [Backups automáticos, histórico e criptografia](./features/backup-automation.md)
- [Restauração seletiva e modo substituir](./features/selective-restore.md)
- [Integridade e versões do backup](./features/backup-integrity.md)
- [Exportar favoritos para outros navegadores](./features/bookmark-export.md)
- [Abas fechadas recentemente](./features/closed-tabs.md)
- [Recuperação automática de sessão](./features/recovery.md)
- [Downloads retomáveis e hashes](./features/downloads.md)
- [Painel flutuante de downloads e progresso na barra](./features/downloads-popover.md)
- [Encerramento protegido](features/shutdown.md)
- [Várias janelas do navegador](features/windows.md)
- [Diagnóstico das abas](features/tab-diagnostics.md)
- [Zoom persistente por site](features/site-zoom.md)
- [Página personalizada de nova guia](features/new-tab.md)
- [Notas e sessões na Command Bar](features/command-search.md)
- [Atalhos de teclado entre abas](features/keyboard-shortcuts.md)
- [Controles de mídia das abas](features/tab-media.md)
- [Pop-up de proteção e Auto skip ads](features/auto-skip-ads.md)
- [Correções de falhas e verificação — 2 de novembro de 2026](reviews/2026-11-02/correctness-fixes.md)
- [Reorganizar atalhos rápidos](./features/titlebar-actions.md)
- [Painel flutuante de Ferramentas](features/tools-popover.md)

- [Suporte a WebExtensions do Chrome](features/extensions.md)
- [Modo Leitura Imersivo](features/reader-mode.md)
- [Abas Efêmeras e containers descartáveis](features/ephemeral-tabs.md)
- [Fronteiras de segurança do aplicativo](security/application-boundaries.md)
- [Investigação de privacidade de rede](security/network-privacy-investigation.md)
- [Proteções contra rastreamento](features/tracking-protection.md)
- [Isolamento de conteúdo por workspace](features/workspace-isolation.md)
- [Isolamento AppContainer (planejado)](security/app-container-isolation.md)
- [Proteção planejada contra sites e downloads suspeitos](roadmap/security-protections.md)
