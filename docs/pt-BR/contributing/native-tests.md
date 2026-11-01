# Integração nativa local

A validação é executada localmente sob demanda pelos comandos em [validação](validation.md). O repositório não possui workflow do GitHub Actions. A suíte nativa do Windows exercita a aplicação de release pelo [WebDriver do Tauri](https://v2.tauri.app/develop/tests/webdriver/).

## Execução isolada no Windows

Instale Node 24, Rust estável e os pré-requisitos normais do Tauri; depois execute na raiz do repositório:

```powershell
npm ci
cargo install tauri-driver --version 2.1.0 --locked --root .cache/native-tools
./tools/testing/install-edge-driver.ps1
npm run test:native:build
$env:VELOCE_NATIVE_DRIVER = '.cache/native-tools/edge/msedgedriver.exe'
npm run test:native
```

O instalador do driver Edge baixa a versão correspondente ao runtime WebView2 instalado pelo endereço de distribuição da Microsoft. Builds nativos usam `.cache/native-target` e o identificador `org.veloce.browser.native-tests`, separando os dados da aplicação dos dados do navegador normal. O executor verifica o identificador em execução antes de alterar estado. Não substitua pelo executável de produção. O perfil de teste persiste entre execuções, mas suas abas contêm somente páginas artificiais de loopback; o perfil nunca entra em commits. As portas padrão são 4444/4445; ajuste `VELOCE_DRIVER_PORT` para outro par livre quando necessário.

A suíte usa um servidor HTTP local e verifica troca de abas residentes sem perda de formulário/rolagem, voltar/avançar em SPA, proteção contra hibernação, atualização do shield no Windows, rascunhos de endereço, foco/Escape em diálogos, redimensionamento e restauração de sessão. Usa comandos existentes e handlers reais do DOM, sem permissões IPC exclusivas de teste. Um pequeno campo de diagnóstico identifica a configuração da aplicação sem expor dados de navegação.

O executor direto também aceita instalações Linux de `tauri-driver`/WebKitWebDriver pelas variáveis `VELOCE_TAURI_DRIVER` e `VELOCE_NATIVE_DRIVER`; esse caminho ainda não foi validado localmente. O macOS precisa de um serviço WebDriver embarcado separado. Transições de DPI entre monitores, gestos de arraste, latência de renderização e memória agregada dos processos ainda precisam de validação específica.

## Medições

Após compilar uma release de produção, execute `node tools/testing/release-metrics.mjs` para registrar revisão Git, versões das ferramentas, bytes brutos/gzip do frontend e bytes do executável nativo. São medições de artefatos, não benchmarks de RAM ou inicialização. Consulte [validação de desempenho](performance.md) para comparações. `npm run security:check` verifica caminhos rastreados/preparados, assinaturas comuns de segredos, regras de exclusão e espaços inválidos sem imprimir o conteúdo dos segredos.
