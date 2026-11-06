# Captura integral em PNG: investigação de viabilidade

**Estado:** a prova do Windows compila, mas a validação em execução está bloqueada antes de o aplicativo de teste abrir. A validação em execução no Linux está pendente.

## Descobertas até o momento

- O produto ainda não tem recurso ou comando de captura integral. Os WebViews de conteúdo podem ser acessados pelo handle nativo do Tauri.
- `CapturePreview` do WebView2 captura o que está sendo exibido no WebView. O WebView2 também expõe o Chrome DevTools Protocol por `CallDevToolsProtocolMethod`; `Page.captureScreenshot` aceita `captureBeyondViewport`, mas esse parâmetro é marcado como experimental e o protocolo não garante compatibilidade entre versões. Consulte [DevTools Protocol no WebView2](https://learn.microsoft.com/en-us/microsoft-edge/webview2/how-to/chromium-devtools-protocol), [API do WebView2](https://learn.microsoft.com/en-us/microsoft-edge/webview2/reference/win32/icorewebview2?view=webview2-1.0.664.37) e [Page.captureScreenshot no CDP](https://chromedevtools.github.io/devtools-protocol/tot/Page/).
- O WebKitGTK oferece `get_snapshot` com a região `FULL_DOCUMENT`. O repositório fixa o binding Rust `webkit2gtk` na versão 2.0.2, que expõe essa região e esse método. Consulte a [API de snapshot do WebKitGTK](https://webkitgtk.org/reference/webkit2gtk/stable/method.WebView.get_snapshot.html).
- Essas APIs tornam plausível capturar o documento inteiro nos dois motores, mas ainda não comprovam o comportamento em páginas longas, elementos fixos/sticky, recursos lazy, quadros ou imagens muito grandes.

## Prova reproduzível

O cenário nativo isolado `capture-probe` abre um fixture local com documento longo, cabeçalho e rodapé fixos, conteúdo sticky, imagem lazy, iframe longo, texto que muda e marcador no final. O comando de prova, protegido por identificador do aplicativo, captura o WebView de conteúdo ativo, grava `artifacts/full-page-probe.png` e informa dimensões, tamanho em bytes, motor e tempo. O cenário verifica que a imagem ultrapassa a altura da viewport e que a posição de rolagem e a altura do documento permanecem iguais.

No Windows, o protótipo chama `Page.captureScreenshot` com `captureBeyondViewport: true`. No Linux, chama o snapshot `FullDocument` do WebKitGTK. O comando rejeita qualquer identificador de aplicativo diferente de `org.veloce.browser.native-tests` e grava apenas no diretório `.cache/native-runs` daquele cenário. `cargo check --manifest-path src-tauri/Cargo.toml --lib` e a compilação isolada do aplicativo de teste no Windows passaram. O cenário em execução não criou uma sessão WebDriver: o EdgeDriver indicou que estava pronto, mas a solicitação para criar a sessão ficou aguardando antes de o processo isolado do aplicativo aparecer. Nenhum PNG ou resultado de cobertura visual foi coletado; portanto, a viabilidade no Windows ainda não foi confirmada.

Para repetir a validação em execução no Windows, indique o EdgeDriver compatível com a versão do Edge instalada e rode o cenário isolado:

```powershell
$env:VELOCE_NATIVE_DRIVER = '.cache/native-tools/edge/msedgedriver.exe'
npm.cmd run test:native -- --case capture-probe
```

## Validação no Linux

O ambiente atual tem somente o alvo Rust Windows, então o comportamento em execução do Linux ainda não foi confirmado. Em uma máquina Linux com os pré-requisitos Tauri/WebKitGTK do projeto, registre a versão ligada e execute:

```sh
pkg-config --modversion webkit2gtk-4.1
npm run test:native:build:fast
npm run test:native -- --case capture-probe
```

A saída do cenário inclui dimensões medidas, tamanho do PNG e tempo. Inspecione `artifacts/full-page-probe.png` no diretório `.cache/native-runs/...` impresso para verificar duplicação de elementos fixos/sticky, imagem lazy, cobertura do iframe e presença do marcador inferior. Informe a versão do WebKitGTK e a saída do cenário antes de fechar o gate multiplataforma.

## Gate de decisão

Com os resultados de Windows e Linux, comparar cobertura, preservação da rolagem, dimensões, impacto de memória e latência. Se a rota CDP do Windows ou o snapshot Linux tiver lacunas relevantes, apresentar alternativas antes de planejar o recurso do produto. Esta investigação não adiciona uma interface de captura para o usuário.
