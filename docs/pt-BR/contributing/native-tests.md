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

O instalador do driver Edge acompanha o runtime WebView2 instalado. As builds usam `.cache/native-target` e o identificador `org.veloce.browser.native-tests`. Cada cenário recebe um diretório novo em `.cache/native-runs/`, com pastas separadas `data`, `profile`, `config`, `cache`, `logs` e `artifacts`. Reinícios dentro do cenário reutilizam esses caminhos. Somente o identificador de testes aceita `VELOCE_NATIVE_TEST_RUN_DIR`, restrito a essa raiz; a versão de produção o ignora. O executor verifica identidade e diretório de dados antes de alterar estado.

Use `npm run test:native -- --list` para listar os cenários ou `npm run test:native -- --case notes` para executar um. Sem argumentos, todos são executados de forma independente, continuando após falhas. Seletores estáveis `data-testid` independem do idioma da interface. As portas padrão são 4444/4445; use `VELOCE_DRIVER_PORT` para outro par livre.

A limpeza executa em `finally`, encerra somente a árvore do processo de driver do cenário e remove data/profile/config/cache/logs mesmo após falha. Relatórios e capturas de falha permanecem em `artifacts`. Erros de limpeza são registrados sem substituir a falha original. Defina `$env:VELOCE_NATIVE_FAIL_CASE = 'notes'` para provocar uma falha após o cenário e verificar a limpeza; remova a variável depois. `VELOCE_NATIVE_SCREENSHOTS=1` também preserva uma captura das atualizações no cenário de diálogos.

A suíte usa páginas HTTP locais e verifica preservação de abas/formulário/rolagem, histórico SPA, shield, rascunhos de endereço, diálogos, resize e restauração de sessão. Também exercita sessões nomeadas, codificação de buscas, notas, fechamento em lote protegido, substituição/fechamento/restauração da divisão e pedidos nativos de permissão. Processos de teste no Windows recebem `--use-fake-device-for-media-stream`; verificações de microfone usam um dispositivo simulado. Cenários de permissões cobrem decisões temporárias/lembradas, redefinição, cancelamento ao navegar, origens incorporadas e persistência após reinício. Os testes usam comandos existentes e handlers reais do DOM, sem permissões IPC exclusivas de teste.

As verificações de regressão do seletor de workspaces registram transições de visibilidade das páginas nativas: abertura, fechamento, reabertura rápida e reconciliação com limite baixo de residentes devem preservar a página por trás do seletor, incluindo os dois painéis da divisão. Valores de formulário, rolagem e contagens de carregamento dos documentos devem permanecer iguais. Diálogos da interface ainda devem ocultar e restaurar ambos os painéis.

O executor direto também aceita instalações Linux de `tauri-driver`/WebKitWebDriver pelas variáveis `VELOCE_TAURI_DRIVER` e `VELOCE_NATIVE_DRIVER`; esse caminho ainda não foi validado localmente. O macOS precisa de um serviço WebDriver embarcado separado. Transições de DPI entre monitores, gestos de arraste, latência de renderização e memória agregada dos processos ainda precisam de validação específica.

## Medições

Após compilar uma release de produção, execute `node tools/testing/release-metrics.mjs` para registrar revisão Git, versões das ferramentas, bytes brutos/gzip do frontend e bytes do executável nativo. São medições de artefatos, não benchmarks de RAM ou inicialização. Consulte [validação de desempenho](performance.md) para comparações. `npm run security:check` verifica caminhos rastreados/preparados, assinaturas comuns de segredos, regras de exclusão e espaços inválidos sem imprimir o conteúdo dos segredos.
