# Guia de Arquitetura e Desenvolvimento de Extensões no Linux

Este guia fornece especificações técnicas precisas, arquitetura e instruções práticas para implementar e testar extensões do Chrome (WebExtensions) no Linux dentro do **Veloce Browser**.

---

## 1. Visão Geral da Arquitetura: Windows vs Linux

O Veloce Browser utiliza os motores nativos de webview de cada sistema para manter alta velocidade e baixo consumo de recursos, sem precisar empacotar uma distribuição completa do Chromium:

| Dimensão | Windows (`#[cfg(windows)]`) | Linux (`#[cfg(target_os = "linux")]`) |
| :--- | :--- | :--- |
| **Motor** | Microsoft Edge WebView2 (baseado em Chromium) | WebKitGTK (baseado em WebKit, similar ao Safari/Epiphany) |
| **API de Extensão** | Nativa via `ICoreWebView2Profile7::AddBrowserExtension` | Modelo de extensão separado por processos do WebKitGTK |
| **Execução em Background** | Tratada nativamente pelo runtime do Chromium | Worker em background injetado / processo web do WebKit |
| **Injeção de Content Scripts** | Automática via perfil do Chromium | `WebKitUserContentManager` / `WebKitWebExtension` |

O backend do Veloce é desacoplado através da abstração do módulo `crate::extensions::platform`, garantindo compilação multiplataforma limpa e zero erros em ambos os sistemas.

---

## 2. Detalhes da Arquitetura do WebKitGTK

O WebKitGTK isola a execução web em processos distintos do sistema operacional:
1. **Processo de Interface / Shell (Veloce e Backend Rust):** Gerencia janelas, abas, banco SQLite, barra de títulos e entrada do usuário.
2. **Processo Web (`WebKitWebProcess`):** Renderiza o HTML, executa o JavaScript das páginas e gerencia as árvores DOM.

Devido a essa fronteira, as extensões no Linux operam por meio de duas estratégias complementares:

```
+----------------------------------------------------------------------------------+
|                            Processo de UI Linux (Veloce)                         |
|                                                                                  |
|  - Armazenamento de Extensões (app_data_dir/extensions/<id>)                    |
|  - Banco de Dados SQLite (installed_extensions)                                  |
|  - Ícone de Ação na Barra de Título & Container de Popup                        |
|  - WebKitUserContentManager (gerencia scripts e estilos de usuário)              |
+----------------------------------------+-----------------------------------------+
                                         | IPC (D-Bus / Ponte WebKit)
+----------------------------------------v-----------------------------------------+
|                       WebKitWebProcess (Renderizador / Aba)                      |
|                                                                                  |
|  - DOM da Janela (Contexto da página)                                            |
|  - Content Scripts Injetados (correspondentes às URLs do manifest)               |
|  - Shim do Runtime chrome.* (chrome.runtime, chrome.storage, chrome.tabs)        |
+----------------------------------------------------------------------------------+
```

---

## 3. Roteiro Recomendado de Implementação no Linux

Ao iniciar o desenvolvimento no Linux, siga este roteiro incremental:

### Fase 1: Injeção de Content Scripts & UserStyles (Alto Valor Imediato)
1. Ler os `content_scripts` do `manifest.json` analisado:
   * Extrair `matches` (ex: `*://*.youtube.com/*`).
   * Extrair os arquivos `js` e `css`.
   * Extrair `run_at` (`document_start`, `document_end`, `document_idle`).
2. Ao criar a webview WebKit para uma aba em `src-tauri/src/window/webview_manager.rs`, acessar seu `WebKitUserContentManager`:
   * Usar `webkit_user_script_new` para registrar scripts com `WEBKIT_USER_CONTENT_INJECT_ALL_FRAMES`.
   * Usar `webkit_user_style_sheet_new` para injeção de CSS.
3. Isso viabiliza imediatamente extensões visuais, modificadores de tema (como Dark Reader) e injeções de script de bloqueio.

### Fase 2: Popups de Ação & Interface
1. Se a extensão declarar uma ação (`action` ou `browser_action`) com `default_popup`:
   * A interface do Veloce renderiza o ícone de ação em `TitlebarActions.svelte`.
   * O clique no botão abre um popover interno (`ExtensionActionPopover.svelte`) carregando `file:///<extension_path>/<popup.html>` ou uma webview interna do Tauri.

### Fase 3: Biblioteca Compartilhada WebKitWebExtension (`.so`)
Para extensões que necessitam de scripts de background e troca completa de mensagens `chrome.runtime`:
1. Criar um crate de biblioteca dinâmica `veloce-web-extension` compilando para `libveloce_extension.so`.
2. Implementar `webkit_web_extension_initialize_with_user_data`.
3. Apontar o contexto do WebKit para o diretório usando `webkit_web_context_set_web_extensions_directory`.
4. Expor uma ponte JavaScript injetando `window.chrome = { runtime: { ... }, storage: { ... } }`.

---

## 4. Configuração do Ambiente e Dependências no Linux

Ao alternar para seu ambiente Linux (Ubuntu / Debian / Fedora / Arch):

### Ubuntu / Debian:
```bash
sudo apt update
sudo apt install -y \
  libwebkit2gtk-4.1-dev \
  libgtk-3-dev \
  libsoup-3.0-dev \
  libjavascriptcoregtk-4.1-dev \
  build-essential \
  curl \
  pkg-config \
  libssl-dev
```

### Fedora:
```bash
sudo dnf install -y \
  webkit2gtk4.1-devel \
  gtk3-devel \
  libsoup3-devel \
  openssl-devel
```

### Arch Linux:
```bash
sudo pacman -S --needed \
  webkit2gtk-4.1 \
  gtk3 \
  libsoup3 \
  base-devel
```

---

## 5. Principais Arquivos no Código-Fonte

* **Implementação de plataforma para Linux:**
  [linux.rs](file:///d:/code/ProjetosCode/projetos/navegador/src-tauri/src/extensions/linux.rs)
* **Serviço Compartilhado de Extensões:**
  [mod.rs](file:///d:/code/ProjetosCode/projetos/navegador/src-tauri/src/extensions/mod.rs)
* **Parser de Manifest (multiplataforma):**
  [manifest.rs](file:///d:/code/ProjetosCode/projetos/navegador/src-tauri/src/extensions/manifest.rs)
* **Repositório de Banco de Dados:**
  [extensions_repo.rs](file:///d:/code/ProjetosCode/projetos/navegador/src-tauri/src/database/extensions_repo.rs)
* **Gerenciador e Popover no Frontend:**
  [ExtensionsModal.svelte](file:///d:/code/ProjetosCode/projetos/navegador/src/lib/components/extensions/ExtensionsModal.svelte)

---

## 6. Comandos de Verificação no Linux

Assim que estiver no ambiente Linux, execute:
```bash
# 1. Verificar compilação Rust
cargo check --manifest-path src-tauri/Cargo.toml

# 2. Executar testes unitários
cargo test --manifest-path src-tauri/Cargo.toml

# 3. Executar checagens de frontend
npm run check

# 4. Iniciar navegador em desenvolvimento
npm run tauri dev
```

