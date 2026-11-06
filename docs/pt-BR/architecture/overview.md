# 🏗️ Arquitetura e Stack Tecnológica

> 🌍 **Idioma:** [English (Main)](../../en/architecture/overview.md) | **Português (pt-BR)**

## 1. Core / Backend (100% Rust)
- **Framework Base:** **Tauri v2** + **Wry (Multi-WebView)** + **Tao**
  - Usa o WebView do sistema. Tamanho e desempenho variam conforme o build e o sistema operacional.
- **Efeitos Visuais Nativos:** `window-vibrancy`
  - Ativa transparência borrada acelerada por hardware (**Mica / Acrylic** no Windows 10/11) com custo zero de CPU.
- **Privacy Shield:** verificações de navegação em Rust e uma lista compartilhada de hosts conhecidos.
  - Um script de página filtra fetch/XHR/sendBeacon e aplica CSS. A implementação atual não inclui adblock-rust nem interceptação completa da rede.
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
   - Cada aba desperta possui sua própria WebView filha, com estado do documento e histórico nativo de voltar/avançar. As instâncias usam o motor do sistema e compartilham o perfil normal do navegador; isso não implica perfis de cookies ou processos do sistema separados.
   - Sincronização em tempo real: quando a barra lateral expande ou recolhe, o Rust redimensiona dinamicamente a área do site em sincronia com a animação.

## 4. Ciclo de vida e persistência

A seleção normal mostra a WebView existente da aba sem navegar novamente. Hibernar ou fechar destrói somente a instância afetada. Abas restauradas em segundo plano são criadas sob demanda. Alguns diálogos ocultam temporariamente as páginas para evitar conflitos de sobreposição nativa. As configurações usam uma camada nativa transparente que projeta os controles da interface e encaminha ações autenticadas aos controladores existentes; a página, incluindo os dois painéis da divisão, permanece visível. O seletor nativo transparente de workspaces também mantém o conteúdo visível durante as animações de abertura e fechamento sem recarregar os documentos.

Somente o callback de carregamento concluído confirma URLs e histórico, usando o ID da aba de origem mesmo quando outra aba já está ativa. Navegações rejeitadas não substituem o endereço confirmado. A gravação da sessão é atômica e participa da transação de exclusão de workspace; falhas do banco são informadas.

Alterar o escudo recria as WebViews porque seus scripts de inicialização são imutáveis. A página ativa recarrega, abas em segundo plano hibernam e o estado das páginas e o histórico de voltar/avançar reiniciam. Essa operação explícita é explicada junto aos controles.
