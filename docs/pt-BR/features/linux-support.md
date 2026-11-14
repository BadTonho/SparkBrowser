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
| Zoom por site | Zoom da página e rastreamento nativo de mudanças | Zoom aplicado na navegação; mudanças nativas por pinça ou Ctrl+roda não são gravadas |
| Controle de mídia por quadro, volume da aba, Picture-in-Picture | Contextos de execução do Chromium DevTools Protocol | Indisponível (informado como `media.unavailable`) |

A URL do shell nos builds de release é `tauri://localhost`. Esquemas personalizados têm caminho vazio e origem opaca em `url::Url`, então as verificações do shell comparam esquema, host e porta em vez de `Url::origin`.

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
```

Limite conhecido: o cenário `backup-identities` não alcança abas de contêiner de conta, porque cada contêiner usa seu próprio diretório de dados do WebKit, que a sessão do WebDriver não automatiza.
