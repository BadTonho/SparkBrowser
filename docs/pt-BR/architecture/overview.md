# 🏗️ Arquitetura e Stack Tecnológica

> 🌍 **Idioma:** [English (Main)](../../en/architecture/overview.md) | **Português (pt-BR)**

## 1. Core / Backend (100% Rust)
- **Framework Base:** **Tauri v2** + **Wry (Multi-WebView)** + **Tao**
  - Usa o motor nativo do sistema (WebView2 / Chromium no Windows, WebKit no macOS/Linux), gerando um executável de apenas **3 MB a 5 MB** e inicialização instantânea.
- **Efeitos Visuais Nativos:** `window-vibrancy`
  - Ativa transparência borrada acelerada por hardware (**Mica / Acrylic** no Windows 10/11) com custo zero de CPU.
- **Bloqueador Nativo de Anúncios e Rastreadores:** `adblock` (`adblock-rust`, o mesmo motor open-source usado pelo Brave)
  - Intercepta e bloqueia anúncios e trackers diretamente no Rust antes mesmo de a página renderizar.
- **Persistência Local:** `rusqlite` (SQLite embarcado)
  - Armazenamento local ultrarrápido para histórico, favoritos, espaços de trabalho e restauração de sessão.

---

## 2. Interface e Motor de Animações (Frontend da UI)
- **Framework UI:** **Svelte (com TypeScript)**
  - Compilado diretamente para instruções nativas enxutas (sem *Virtual DOM*), mantendo o consumo de memória no mínimo absoluto.
- **Motor de Animações:** `svelte/motion` (`spring`, `tweened`) + `svelte/transition` + **CSS Moderno**
  - Animações com física de mola real a **60/120 FPS**, transições suaves de abas, micro-interações e *glassmorphism*.

---

## 3. Composição da Janela com Multi-WebView

```
+-----------------------------------------------------------------------+
|  Janela Principal (Rust + UI Svelte + Fundo Nativo Mica/Acrylic)      |
|  +-----------------+  +--------------------------------------------+  |
|  |                 |  |                                            |  |
|  |  Barra Lateral  |  |  Quadro do Site (WebView Filha no Rust)    |  |
|  |  e Abas         |  |  Renderiza o site ativo (ex: YouTube)      |  |
|  |  Animadas       |  |  Redimensiona junto com a animação da UI   |  |
|  |                 |  |                                            |  |
|  +-----------------+  +--------------------------------------------+  |
+-----------------------------------------------------------------------+
```

1. **Camada Base (Janela Principal em Rust + Svelte):**
   - Renderiza a barra de título customizada, **Barra Lateral (Sidebar)** retrátil, controles de navegação, barra de URL e favoritos sobre um fundo de vidro fosco nativo.
2. **Quadro de Conteúdo Web (Multi-WebView controlada pelo Rust):**
   - Os sites abertos rodam em instâncias isoladas de WebView encaixadas dentro da moldura do navegador.
   - Sincronização em tempo real: quando a barra lateral expande ou recolhe, o Rust redimensiona dinamicamente a área do site em sincronia com a animação.
