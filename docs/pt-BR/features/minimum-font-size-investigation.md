# Tamanho mínimo de fonte: investigação de viabilidade

**Estado:** a prova isolada compila. O EdgeDriver compatível inicia, mas o harness não conclui a criação da sessão WebDriver, então não há medições visuais. A validação Linux está preparada para uma execução posterior.

## Descobertas

- O WebKitGTK expõe uma configuração nativa `minimum-font-size`, em pixels. A documentação alerta que um valor diferente de zero pode quebrar layouts. Consulte [`Settings.set_minimum_font_size`](https://webkitgtk.org/reference/webkit2gtk/stable/method.Settings.set_minimum_font_size.html) e a propriedade [`minimum-font-size`](https://webkitgtk.org/reference/webkit2gtk/stable/property.Settings.minimum-font-size.html).
- A interface Win32 documentada `ICoreWebView2Settings` inclui controle de zoom, mas não uma configuração de tamanho mínimo de fonte. Essa conclusão se limita à interface documentada; não afirma que inexista qualquer alternativa não documentada no runtime. Consulte a [referência de ICoreWebView2Settings](https://learn.microsoft.com/en-us/microsoft-edge/webview2/reference/win32/icorewebview2settings?view=webview2-1.0.4022.49).
- O app oferece zoom da página, mas ele escala a página inteira, inclusive textos que já estão acima de um mínimo escolhido, e pode alterar o layout responsivo. Portanto, não equivale a um tamanho mínimo de fonte.
- A prova isolada `minimum-font-probe` inclui textos de 8, 12, 16 e 24 px, texto relativo, link, botão, campo de entrada e iframe. Ela captura zoom em 100% e 150%. No Linux, também compara as configurações do WebKitGTK em 0 px e 12 px e restaura o valor anterior.

## Resultado no Windows

Este ambiente tem WebView2 Runtime `157.0.4341.42` e inicialmente tinha EdgeDriver em cache `156.0.4314.58`. A Microsoft exige versões correspondentes para automatizar apps WebView2. Um EdgeDriver compatível `157.0.4341.42` foi baixado para o cache ignorado dos testes e seu endpoint `/status` informou que estava pronto. Consulte [Automatizar e testar apps WebView2 com Microsoft Edge WebDriver](https://learn.microsoft.com/en-us/microsoft-edge/webview2/how-to/webdriver).

A prova compila com `npm.cmd run test:native:build:fast`. Mesmo com o driver correspondente, o harness não concluiu a criação da sessão WebDriver; o app de teste e as capturas não chegaram a ser produzidos antes de a tentativa ser interrompida. Ainda não há conclusão visual sobre o zoom no WebView2. O comando com o driver compatível é:

```powershell
$env:VELOCE_NATIVE_DRIVER = "$PWD/.cache/native-tools/edge/157/msedgedriver.exe"
npm.cmd run test:native -- --case minimum-font-probe
```

## Validação no Linux

Registre a versão instalada do WebKitGTK e execute a prova isolada:

```sh
pkg-config --modversion webkit2gtk-4.1
npm run test:native:build:fast
VELOCE_TAURI_DRIVER="$(command -v tauri-driver)" VELOCE_NATIVE_DRIVER="$(command -v WebKitWebDriver)" VELOCE_DRIVER_PORT=4484 npm run test:native -- --case minimum-font-probe
```

Inspecione `minimum-font-probe.json` e as capturas no diretório `artifacts` da execução. Compare 0 px e 12 px para ver se a configuração nativa afeta os tamanhos abaixo do limite; confira também os controles, o iframe e as capturas com zoom em 100% e 150%. A prova restaura a configuração anterior após a medição.

## Recomendação e gate de decisão

O suporte documentado dos motores é assimétrico: o WebKitGTK tem uma configuração nativa de tamanho mínimo de fonte, mas a interface WebView2 documentada não tem equivalente. O zoom é uma substituição imperfeita, e reescrever CSS está fora desta investigação. Não implementar uma preferência multiplataforma de tamanho mínimo antes de repetir a prova WebView2 com um driver compatível e revisar as capturas Linux. Se o Windows continuar sem equivalente nativo suportado, decidir entre aceitar uma limitação por plataforma ou definir uma alternativa em outro escopo antes de implementar.
