# Suporte ao Linux

O Spark roda no Windows (WebView2) e no Linux (WebKitGTK por meio do `webkit2gtk` 2.0). As duas plataformas compartilham a mesma interface Svelte e o mesmo estado em Rust; o código específico fica atrás de `#[cfg(windows)]` e `#[cfg(target_os = "linux")]`.

## Paridade nativa

| Área | Windows | Linux |
|---|---|---|
| Proteção do shell (só documentos do shell em `main`) | WebView2 `NavigationStarting` | Guarda `on_navigation` de plugin do Tauri (`security/linux.rs`) |
| Novas janelas e DevTools em views internas | Negadas / só em desenvolvimento | Negadas / só em desenvolvimento |
| Barra de endereço e voltar/avançar (inclusive navegação SPA) | Eventos de origem e histórico do WebView2 | `notify::uri`, lista de histórico e eventos de carregamento do WebKitGTK |
| Foco do painel no split view | WebView2 `GotFocus` | GTK `focus-in-event` |
| Mudo da aba e indicador de áudio | WebView2 `IsMuted` / `IsDocumentPlayingAudio` | WebKitGTK `is-muted` / `is-playing-audio` |
| Zoom por site | Zoom da página e rastreamento nativo de mudanças | Zoom salvo aplicado quando a página é confirmada; Ctrl+roda muda o zoom e o grava (`zoom/linux.rs`) |
| Ícones dos sites (favicons) | WebView2 `FaviconChanged` / `GetFavicon` | WebKitGTK `notify::favicon` com o banco de favicons ativo por perfil; gravado como um PNG pequeno (`window/favicon/linux.rs`) |
| Bloqueio de rede (escudo) | Manipulador de requisições do WebView2 | Listas de filtros compiladas em uma lista de regras de conteúdo do WebKit por página (`privacy/content_filter_linux.rs`) |
| Permissões de site (câmera, microfone, localização, notificações) | Lembradas no perfil do WebView2 | `permission-request` do WebKitGTK; escolhas gravadas no banco do perfil (`site_permissions/linux.rs`) |
| Progresso e cancelamento de downloads | Operações de download do WebView2 | Objeto de download do WebKitGTK acompanhado a partir do evento de download do motor (`downloads/native_linux.rs`) |
| Credenciais e cookies do widget do Gmail | Gerenciador de Credenciais do Windows, gerenciador de cookies do WebView2 | Secret Service por `secret-tool`, gerenciador de cookies do WebKitGTK |
| Memória da aba | Lista de quadros por processo do WebView2 | Memória residente dos processos filhos `WebKitWebProcess`, dividida entre as abas residentes |
| Volume da aba (0-200%) | Chromium DevTools Protocol e captura de stream para amplificação | Script da página por um manipulador de mensagens do WebKit; atenuação pelo volume do elemento, amplificação por WebAudio |
| Player flutuante Picture-in-Picture | Stream capturado do vídeo, imagem original oculta | Quadros copiados para um canvas no popup; a origem continua visível |

### Mídia no WebKitGTK

O WebKitGTK não tem `HTMLMediaElement.captureStream()` e, em builds comuns, não tem WebRTC, então um `MediaStream` não pode ser exibido em um `<video>`. Os caminhos do Linux diferem do Windows assim:

- **Volume acima de 100%** passa o elemento por um nó de ganho do WebAudio. Uma fonte de elemento de mídia nunca pode ser liberada, então o grafo permanece enquanto o documento existir (o ganho volta a 1 em 100%). Só é usado para mídia que o WebAudio pode ler (mesma origem, `blob:` ou CORS); caso contrário o ajuste informa `limited` e o áudio da página não é alterado.
- **Picture-in-Picture** funciona com qualquer vídeo sem proteção, inclusive de outra origem, porque o popup desenha os quadros com `drawImage`. O vídeo original continua visível porque o WebKit pausa vídeos mudos com autoplay que ficam invisíveis. A janela sem moldura é criada como webview relacionada e o WebKit refaz seu documento depois de `window.open()`, então o player é montado depois disso.
- **Comportamento da janela:** sempre no topo, ocultar da barra de tarefas e a sombra são feitos na medida do possível e dependem do gerenciador de janelas (o Wayland ignora o sempre no topo em janelas comuns).
- **Eco do motor:** o WebKitGTK informa o volume lembrado do stream de áudio como `volumechange`. O controlador ignora essas mudanças, a menos que o usuário tenha interagido com a página, para que o volume da aba nunca vire o volume próprio da página.

A URL do shell nos builds de release é `tauri://localhost`. Esquemas personalizados têm caminho vazio e origem opaca em `url::Url`, então as verificações do shell comparam esquema, host e porta em vez de `Url::origin`.

### Diferenças que continuam no Linux

- **Contadores do escudo:** o WebKit aplica as regras de conteúdo sem informar cada requisição bloqueada, então o contador só conta navegações bloqueadas. As regras são ligadas à página quando ela começa a carregar e só nas páginas que o escudo protege; uma exceção de site vale a partir do próximo carregamento. A limpeza de parâmetros de rastreamento em `GET` e o cabeçalho `Sec-GPC` exigem reescrever a requisição e não estão disponíveis.
- **Downloads:** o WebKitGTK não pausa nem retoma uma transferência, então essas ações ficam ocultas; cancelar e o progresso funcionam.
- **Permissões:** captura de tela e outros tipos de pedido são negados. O WebKit não informa o quadro que pediu, então o diálogo mostra a origem da página.
- **Atualização automática:** só no Windows. No Linux a instalação vem do gerenciador de pacotes (deb, rpm) ou de um novo AppImage.
- **Widget de localização:** indisponível; o WebKitGTK não tem API de localização do sistema sem o GeoClue.
- **Aparência da janela:** Mica/Acrylic não têm equivalente no Linux, e o GTK registra `gtk_window_set_titlebar() called on a realized window`, o que é inofensivo.

## Requisitos

Instale os pacotes de desenvolvimento do WebKitGTK 4.1 e do GTK 3 da sua distribuição (veja o [guia de extensões no Linux](../architecture/linux-extensions-guide.md)). O `WebKitWebDriver` é necessário para os testes nativos.

## Validação no Linux

```bash
npm install
npm run validate          # arquitetura, testes, check, fmt, clippy -D warnings, cargo test, build, segurança
npm run tauri dev         # execução manual
```

Os testes nativos também precisam do `tauri-driver` no diretório ignorado `.cache/native-tools`:

```bash
cargo install tauri-driver --locked --root .cache/native-tools
npm run test:native:build:fast
npm run test:native:critical
npm run test:native -- --case media-linux   # volume, mudo e Picture-in-Picture no WebKitGTK
npm run test:native -- --case keyboard-linux   # Ctrl+L e Ctrl+T apertados a partir de uma página
```

Limite conhecido: o cenário `backup-identities` não alcança abas de contêiner de conta, porque cada contêiner usa seu próprio diretório de dados do WebKit, que a sessão do WebDriver não automatiza.
