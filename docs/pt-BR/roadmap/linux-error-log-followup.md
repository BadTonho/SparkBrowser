# Pendência no Linux: registro de erros

[English](../../en/roadmap/linux-error-log-followup.md)

Situação: **pendente**, a fazer em uma máquina Linux.

## Por quê

O registro de erros ([error-log.md](../features/error-log.md)) grava uma falha que o código Rust antes jogava fora (`let _ = resultado;`) quando esse código chama `crate::error_log::ignored(resultado)`. A conversão foi feita no Windows com `cargo check`, então só o código que compila no Windows pôde ser convertido e conferido. O código que só compila no Linux (`#[cfg(target_os = "linux")]`, `#[cfg(not(windows))]`, `#[cfg(unix)]`, arquivos como `*_linux.rs`) foi deixado como estava para não quebrar o build do Linux.

## O que fazer

1. No Linux, instale as dependências de build do [guia de suporte ao Linux](../features/linux-support.md) e confirme que `cargo check --lib` passa em `src-tauri` antes de mudar qualquer coisa.
2. Em cada ponto da lista abaixo, troque `let _ = EXPR;` por `crate::error_log::ignored(EXPR);` (a instrução pode ocupar várias linhas; mantenha o `);` final).
3. Rode `cargo check --lib` no Linux. Um ponto que não compila é um destes:
   - `EXPR` não é um `Result` (devolve `bool`, `Option`, `()` ou um `JoinHandle`): volte para `let _ = `.
   - O tipo de erro não implementa `Debug` (`` `T` doesn't implement `Debug` ``): volte para `let _ = `, ou converta o erro antes com `.map_err(|e| e.to_string())`.
4. Rode `cargo fmt`, `cargo test --lib` e `npm test`.
5. Confirme que o build do Windows continua compilando (`cargo check --lib` no Windows, ou no CI).
6. Adicione uma linha ao changelog da versão em andamento (`Melhorias`, em inglês e português): erros que o Linux antes ignorava em silêncio agora são registrados.
7. Faça o commit, por exemplo `feat(error-log): log ignored Results in Linux-only code`.

Pontos dentro de outra instrução já convertida (um closure dentro de `ignored(...)`) podem ficar como estão.

## Pontos para conferir

Os números de linha são do commit `b1c365f`. A lista traz todo `let _ = chamada(...)` restante fora dos testes; alguns compilam no Windows e foram deixados porque não são um `Result` ou o erro não tem `Debug`, então espere ter que voltar alguns.

- [`backup/credentials.rs`](../../../src-tauri/src/backup/credentials.rs): 75, 179, 180, 192, 193
- [`backup/sync.rs`](../../../src-tauri/src/backup/sync.rs): 297, 308
- [`commands/capture_probe.rs`](../../../src-tauri/src/commands/capture_probe.rs): 172
- [`commands/dark_mode_probe.rs`](../../../src-tauri/src/commands/dark_mode_probe.rs): 73
- [`commands/media.rs`](../../../src-tauri/src/commands/media.rs): 16, 66
- [`commands/minimum_font_probe.rs`](../../../src-tauri/src/commands/minimum_font_probe.rs): 25
- [`commands/privacy.rs`](../../../src-tauri/src/commands/privacy.rs): 273, 279
- [`commands/site_permissions.rs`](../../../src-tauri/src/commands/site_permissions.rs): 41, 61
- [`commands/updates.rs`](../../../src-tauri/src/commands/updates.rs): 41, 64, 90
- [`commands/whatsapp.rs`](../../../src-tauri/src/commands/whatsapp.rs): 32
- [`downloads/native.rs`](../../../src-tauri/src/downloads/native.rs): 14
- [`downloads/native_fallback.rs`](../../../src-tauri/src/downloads/native_fallback.rs): 21, 105, 122, 133, 135
- [`downloads/native_windows.rs`](../../../src-tauri/src/downloads/native_windows.rs): 565, 569
- [`extensions/windows.rs`](../../../src-tauri/src/extensions/windows.rs): 15
- [`media/linux.rs`](../../../src-tauri/src/media/linux.rs): 27
- [`media/mod.rs`](../../../src-tauri/src/media/mod.rs): 224
- [`media/pip.rs`](../../../src-tauri/src/media/pip.rs): 133
- [`media/protocol_linux.rs`](../../../src-tauri/src/media/protocol_linux.rs): 109
- [`media/visualizer/windows.rs`](../../../src-tauri/src/media/visualizer/windows.rs): 249, 252, 255
- [`media/volume.rs`](../../../src-tauri/src/media/volume.rs): 125, 222
- [`privacy/content_filter_linux.rs`](../../../src-tauri/src/privacy/content_filter_linux.rs): 39, 47
- [`privacy/filter_lists.rs`](../../../src-tauri/src/privacy/filter_lists.rs): 125
- [`security/mod.rs`](../../../src-tauri/src/security/mod.rs): 103
- [`site_permissions/linux.rs`](../../../src-tauri/src/site_permissions/linux.rs): 38, 48, 218
- [`site_permissions/mod.rs`](../../../src-tauri/src/site_permissions/mod.rs): 89, 92
- [`site_permissions/profile.rs`](../../../src-tauri/src/site_permissions/profile.rs): 15
- [`updates/mod.rs`](../../../src-tauri/src/updates/mod.rs): 55
- [`updates/service.rs`](../../../src-tauri/src/updates/service.rs): 213
- [`webview_runtime/mod.rs`](../../../src-tauri/src/webview_runtime/mod.rs): 91
- [`window/browser_windows.rs`](../../../src-tauri/src/window/browser_windows.rs): 36
- [`window/caret_browsing.rs`](../../../src-tauri/src/window/caret_browsing.rs): 22
- [`window/favicon/linux.rs`](../../../src-tauri/src/window/favicon/linux.rs): 66
- [`window/new_tab/widgets/cookies.rs`](../../../src-tauri/src/window/new_tab/widgets/cookies.rs): 124, 128
- [`window/new_tab/widgets/spotify/platform.rs`](../../../src-tauri/src/window/new_tab/widgets/spotify/platform.rs): 100
- [`window/overlay_host.rs`](../../../src-tauri/src/window/overlay_host.rs): 105
- [`window/settings_tab.rs`](../../../src-tauri/src/window/settings_tab.rs): 234
- [`window/tab_drag_preview/surface.rs`](../../../src-tauri/src/window/tab_drag_preview/surface.rs): 58, 60
- [`window/tab_memory.rs`](../../../src-tauri/src/window/tab_memory.rs): 112
- [`window/vibrancy.rs`](../../../src-tauri/src/window/vibrancy.rs): 20

Já se sabe que falham no Windows: `extensions/windows.rs`, `site_permissions/profile.rs` (o erro não tem `Debug`) e `window/overlay_host.rs` (não é um `Result`).

## Também não coberto

- Os scripts da página de nova guia (`public/new-tab/*.js`) ainda têm blocos `catch` vazios. Eles não chamam o registro diretamente; teriam que enviar uma mensagem `error` pela ponte, como os erros não tratados já fazem.
- A notificação só mostra `error` e `critical`. Se quiser uma opção para mostrar também os avisos, é uma mudança pequena em `error_log.rs` (`Level::notifies`).
