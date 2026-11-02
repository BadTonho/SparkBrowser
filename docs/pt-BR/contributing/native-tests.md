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

O cenário `recovery` interrompe seu próprio processo de teste, verifica todas as abas e o foco da divisão e depois confere um reinício normal. `recovery-failure` injeta um trigger SQLite somente no banco do cenário encerrado e com identidade verificada, comprova que a falha de inicialização mantém o snapshot anterior, remove o trigger e tenta novamente. Os cenários `closed-tabs` e `backup` cobrem reabertura e importação revisada com novas tentativas persistentes de preferências.

A suíte também inclui `downloads` (checkpoints duráveis, intervalos HTTP, integridade e controles nativos), `shutdown` (gravação de notas, revisão de novos riscos, cancelamento e falha SQLite), `tab-diagnostics` (residência real e proteção revalidada), `zoom` (escala das páginas residentes, origens, navegação, atalhos, reinício e preferências do backup) e `command-search` (resultados Unicode limitados, seleção estável, URLs originais das notas e detalhes revisados de sessões). Os downloads dos testes gravam exclusivamente em `data/downloads` do cenário, sem usar a pasta pessoal de Downloads. A suíte completa tem vinte e um cenários independentes. No Windows, `keyboard` envia entrada real somente ao executável isolado em primeiro plano e verifica fechamento/reabertura de abas consecutivas mantendo modificadores pressionados; mantenha o desktop de teste disponível. Ela não executa instaladores nem usa VM.

## Medições

Após compilar uma release de produção, execute `node tools/testing/release-metrics.mjs` para registrar revisão Git, versões das ferramentas, bytes brutos/gzip do frontend e bytes do executável nativo. São medições de artefatos, não benchmarks de RAM ou inicialização. Consulte [validação de desempenho](performance.md) para comparações. `npm run security:check` verifica caminhos rastreados/preparados, assinaturas comuns de segredos, regras de exclusão e espaços inválidos sem imprimir o conteúdo dos segredos.

Execute `--case media-mute` para atividade de áudio nativa, mute em segundo plano e valores iniciais após reiniciar.

`--case media-volume` mede o ganho de áudio artificial e verifica volume e fallback na navegação. Também exercita o pop-up de volume com o percentual abaixo do slider pelo identificador da sua própria WebView no WebDriver, verifica as dimensões de 200 × 64 px e a distância de 6 px abaixo do botão, alinhamento e atualização do percentual, setas do teclado, arraste pelo ponteiro, fechamento, reabertura rápida e os dois layouts de abas, e confirma que nem a página nem qualquer painel da divisão são ocultados ou recarregados. As verificações da WebView auxiliar selecionam temporariamente o documento empacotado e restauram o identificador da interface principal; não acrescentam permissões de IPC da aplicação. O helper de redimensionamento Windows altera somente a área cliente do executável isolado verificado, pois o redimensionamento pelo WebDriver pode afetar uma única WebView em vez da janela que a contém. Regressões JavaScript cobrem os limites da janela, preservação do arraste, agrupamento e envio do último valor ao fechar antes do próximo quadro de animação.

`--case media-pip` valida a detecção automática de vídeos sem abrir o volume nem solicitar um relatório manualmente, seleciona o maior vídeo elegível pelo botão da barra e verifica abertura/saída nos dois layouts e no painel focado da divisão. Também cobre o fechamento a partir de outra aba/workspace, fechamento externo, vídeos de outra origem e proteção da WebView de origem. Os comandos internos existentes mantêm a cobertura de documentos antigos, candidatos explícitos e players restritos. Regressões JavaScript verificam desempate estável na seleção, consultas compartilhadas, bloqueio de cliques repetidos e cancelamento quando o contexto focado muda durante uma consulta. O pop-up de volume não contém controles de Picture-in-Picture.

O cenário de PiP também seleciona o identificador WebDriver do player próprio e verifica uma trilha em movimento somente de vídeo, volume no hover, atualização imediata do percentual em 0%, 100% e 200%, ajuste por teclado, foco com Escape, sincronização com o volume nativo e frames de outra origem. Os vídeos sintéticos contêm trilhas de áudio geradas para exercitar a amplificação sem microfones nem mídia externa. Regressões do player cobrem a duração do hover/foco/arraste, confirmações antigas de volume e liberação somente das trilhas espelhadas ao fechar. Campos da sessão própria/nativa distinguem a janela da aplicação da alternativa nativa sem alterar os comandos de entrada/saída.

Depois que o PiP nativo fecha, o cenário ativa somente a janela Windows do aplicativo isolado verificado antes de conferir a retomada da detecção. O foco no DOM pelo WebDriver não garante que a janela do sistema operacional esteja em primeiro plano; as consultas periódicas são pausadas intencionalmente enquanto a aplicação está sem foco.

As verificações do player próprio conferem o layout de 400 × 225 px sem moldura, os controles SVG centralizados e o domínio de origem, capturam imagens com controles visíveis/ocultos e exercitam ocultação por inatividade, foco pelo teclado, visibilidade em pausa, arraste do cabeçalho, minimizar/restaurar e limites em janela pequena. O helper Windows da janela flutuante identifica somente o executável isolado exato e sua janela flutuante antes de inspecionar/restaurar ou testar arraste/redimensionamento nativo pelo ponteiro; a entrada exige foco em primeiro plano e um ponto dentro daquela janela verificada. Testes JavaScript usam um relógio controlado para verificar o prazo de dois segundos e suas exceções de foco, volume e arraste.

A regressão de mudanças de iframe remove um frame não relacionado enquanto outra aba está focada, deixando a chave agregada de mídia da origem invalidada. Ajustes comuns de volume devem rejeitar essa chave; o player flutuante só pode alterar o volume após conferir sua sessão de origem em relatórios novos. Os testes Rust cobrem a proteção da chave invalidada e a rejeição de documentos de origem, gerações e sessões antigos.
