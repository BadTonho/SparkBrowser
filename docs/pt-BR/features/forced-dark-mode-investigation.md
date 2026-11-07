# Modo escuro forçado em páginas: investigação de viabilidade

**Estado:** a prova isolada compila no Windows, mas a validação visual em execução está bloqueada durante a criação da sessão WebDriver. A validação Linux está pendente.

## Descobertas

- A aparência escura existente muda a interface do navegador e as configurações; ela não força o conteúdo das páginas a usar uma paleta escura.
- `PreferredColorScheme` do WebView2 seleciona `dark`, `light` ou `auto` para um perfil. A API define `prefers-color-scheme`; a página precisa fornecer estilos escuros para que suas superfícies mudem. Consulte a [API de perfil do WebView2](https://learn.microsoft.com/en-us/microsoft-edge/webview2/reference/win32/icorewebview2profile).
- O WebKit documenta o mesmo modelo de adesão da página e informa que não escurece automaticamente qualquer conteúdo. Consulte [Suporte a modo escuro no WebKit](https://webkit.org/blog/8840/dark-mode-support-in-webkit/).
- O cenário isolado `dark-mode-probe` cobre uma página com estilos escuros, uma página clara sem estilos escuros e uma página já escura. Cada uma contém imagem SVG local, vídeo gerado em canvas, texto dinâmico e uma variante de inversão CSS exclusiva da prova. O cenário registra cores calculadas e `matchMedia`, e mantém capturas e JSON em `.cache/native-runs/.../artifacts`.
- O comando da prova é restrito ao aplicativo de teste isolado `org.veloce.browser.native-tests` e restaura o valor de esquema de cores anterior. Não foram adicionados preferência, exceção por site, interface ou comportamento público do produto.

## Resultado no Windows

`cargo check --manifest-path src-tauri/Cargo.toml --lib` e `npm.cmd run test:native:build:fast` passam. A validação em execução não chegou ao fixture. Neste ambiente Windows 10.0.26300, o EdgeDriver 156.0.4314.58 informa que está pronto, mas `POST /session` falha com `Microsoft Edge failed to start: crashed (DevToolsActivePort file doesn't exist)` antes de haver WebView de teste ou captura de tela. Portanto, o comportamento visual do WebView2 e da transformação CSS experimental ainda não foi medido aqui.

## Validação no Linux

No Linux com os pré-requisitos Tauri, GTK e WebKitGTK do projeto instalados, registre a versão do WebKitGTK e execute:

```sh
pkg-config --modversion webkit2gtk-4.1
npm run test:native:build:fast
VELOCE_TAURI_DRIVER="$(command -v tauri-driver)" VELOCE_NATIVE_DRIVER="$(command -v WebKitWebDriver)" VELOCE_DRIVER_PORT=4484 npm run test:native -- --case dark-mode-probe
```

A prova Linux altera temporariamente a preferência de tema GTK somente no aplicativo de teste isolado e restaura o valor anterior. O cenário não grava as configurações de aparência do desktop. Inspecione as capturas e `dark-mode-probe.json` no diretório da execução impresso. Confirme que a página compatível reage à preferência escura, que a página clara sem suporte continua clara, que a página escura original permanece legível e que a transformação não degrada as cores de imagens/vídeo nem o conteúdo dinâmico. Registre a versão do WebKitGTK e o caminho dos artefatos junto com os resultados.

## Gate de decisão

Não tratar o sinal nativo de esquema de cores como um modo escuro forçado completo: páginas sem estilos escuros próprios permanecem como foram escritas. Compare os artefatos reais do Windows e Linux antes de escolher a estratégia de implementação. Se a transformação visual prejudicar mídia, controles ou páginas já escuras, documente as limitações e apresente alternativas antes de planejar configurações do produto ou exceções por site.
