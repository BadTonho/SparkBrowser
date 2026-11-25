# Registro de erros e notificações de erro

[English](../../en/features/error-log.md)

O Spark avisa o usuário quando algo dá errado e mantém um registro local, para que bugs possam ser reportados já com os detalhes coletados. O Spark nunca envia nada sozinho: quem escolhe enviar o erro é o usuário.

## Notificação

Quando um erro acontece, um cartão aparece no **canto inferior direito** da janela do navegador (até três por vez, os demais esperam). Ele fica até ser dispensado e tem dois botões:

- **Ver erro** mostra os detalhes dentro do cartão.
- **Enviar erro** abre uma nova aba do Spark na página de **nova issue** do projeto (`https://github.com/BadTonho/SparkBrowser/issues/new`) com título e texto já preenchidos: versão do Spark, sistema, horário, tipo, resumo e detalhes (cortados para caber no limite do endereço). O usuário revisa e envia no GitHub. Se a página não puder ser aberta, o cartão avisa.

O mesmo erro aparece no máximo uma vez por minuto e no máximo cinco cartões aparecem por minuto, então um laço não consegue inundar a tela.

## Níveis

Toda entrada tem um de quatro níveis. Só os dois sérios interrompem o usuário com um cartão; os outros dois apenas vão para o registro.

| Nível | Significado | Exemplos | Cartão |
| --- | --- | --- | --- |
| `info` | Eventos relevantes que não são problemas | início, fechamento normal | não |
| `warning` | Uma falha que o Spark ignorou e da qual se recuperou | um `Result` descartado no Rust, um `.catch` vazio na interface, o início de um travamento | não |
| `error` | Algo falhou e o usuário pode notar | erros nativos, erros de script | sim |
| `critical` | O próprio app está com problema | um panic, um travamento que terminou, uma execução que não fechou normalmente | sim, em vermelho mais forte |

## O que conta como erro

| Tipo | Origem |
| --- | --- |
| `panic` | Um erro interno inesperado no código Rust (em qualquer thread) |
| `hang` | A thread principal não respondeu a um vigia por 10 segundos; é reportado quando ela volta a responder, com a duração |
| `native` | Qualquer erro que o Spark já mostrava pelo evento `browser-error` (downloads, privacidade, operações de abas etc.). Bloqueios do escudo ("Navigation blocked") são intencionais e não são reportados |
| `interface` | Erro de script ou rejeição não tratada na interface do navegador (shell, configurações, ferramentas, downloads) |
| `new-tab` | Erro de script ou rejeição não tratada na página de nova guia (até 3 por carregamento) |
| `shutdown` | Falha ao salvar ou fechar o Spark (erro de gravação, instalador ou reinício). O log tem a falha exata, o motivo e a fase; o diálogo de fechamento mostra só uma mensagem amigável |
| `unclean-exit` | A execução anterior deixou o marcador `session.running`: uma queda, um travamento fechado à força ou falta de energia. Aparece alguns segundos depois de iniciar |

### Contexto de um travamento

Um travamento não tem mensagem de erro, então a entrada `hang` traz um **rastro**: as últimas 40 coisas que o app foi mandado fazer (cada comando da interface e cada abertura ou fechamento de popover), cada uma com hora e thread. A última linha é a causa mais provável. No Windows a entrada também lista **onde cada thread está esperando** (a pilha de chamadas), o que mostra um deadlock diretamente: duas threads esperando cada uma algo que a outra segura. Os nomes das funções aparecem quando o arquivo `.pdb` está ao lado do executável (builds de desenvolvimento); um build de lançamento mostra deslocamentos do módulo. O código está em [`src-tauri/src/hang_stack.rs`](../../../src-tauri/src/hang_stack.rs).

### Comandos que falham e o console

Todo comando da interface que falha é registrado como `warning` (`Command "nome" failed: …`), mesmo quando quem chamou trata a falha. Tudo que a interface imprime com `console.error` também vira `warning`.

### Falhas que eram ignoradas

O código que antes jogava o erro fora agora o registra como `warning`, com o local que o ignorou:

- No Rust, `let _ = resultado;` virou `crate::error_log::ignored(resultado);`, que registra o erro com o arquivo e a linha. Use-o para todo novo `Result` ignorado de propósito.
- Na interface, um `.catch(() => {})` vazio virou `.catch(logIgnored)` (`src/lib/services/ignored-error.ts`). Filas de promessas que só mantêm a cadeia viva mantêm o catch vazio, porque o erro pertence a quem chamou.

Tudo que antes só era impresso no console (`eprintln!`) agora também vai para o log como `warning` (`crate::error_log::warn`), e a página de nova guia registra suas falhas ignoradas por `window.__sparkLogIgnored`. Ainda não coberto: o código Rust que só compila no Linux foi convertido sem ser compilado aqui, os scripts injetados em páginas do Google Drive (`gdrive_bridge.js`) ainda têm catches vazios de melhor esforço, e alguns pontos do Rust transformam um erro em um "falhou" simples sem guardar o texto.

## Arquivo de registro

`logs/spark-errors.log` dentro da pasta de dados do Spark (no Windows, `%APPDATA%\org.spark.browser\logs`). Cada entrada, cada início e cada fechamento normal são acrescentados ali, uma linha por entrada com o nível (`[INFO]`, `[WARN]`, `[ERROR]`, `[CRITICAL]`). A mesma entrada é escrita no máximo uma vez a cada 10 segundos, exceto `critical`; a próxima entrada gravada para ela informa quantas repetições foram ignoradas (`repeated N more times since the last entry`). Em 512 KB ele é renomeado para `spark-errors.log.old`, então nunca usa mais de cerca de 1 MB. Janelas privadas não escrevem nele nem mostram esses cartões.

## Ver o log no aplicativo

**Configurações → Diagnósticos locais** lista as últimas entradas do log (até 150, da mais nova para a mais antiga, lidas do fim do `spark-errors.log`), cada uma com o nível, a hora (UTC) e o texto completo com os detalhes. Um filtro escolhe um nível, **Copiar** coloca as entradas listadas na área de transferência para colar em um relatório, e o caminho do arquivo de log aparece no fim. Nada é enviado. Os contadores internos de desempenho (tempos das operações e dos comandos da interface) ficam em um bloco recolhido, "Desempenho interno", logo abaixo. O comando `get_recent_errors` está em [`src-tauri/src/commands/error_report.rs`](../../../src-tauri/src/commands/error_report.rs).

## Privacidade

Os relatos são redigidos antes de serem registrados ou mostrados: a pasta do usuário vira `~` e qualquer endereço `http(s)://` é cortado depois do host (`https://example.test/…`). Resumos são limitados a 300 caracteres e detalhes a 6.000.

## Código

- [`src-tauri/src/error_log.rs`](../../../src-tauri/src/error_log.rs): registro, redação, limite de frequência, panic hook, vigia de travamento e marcador de execução. `install` roda em `run()` antes de qualquer janela abrir, `watch` no `setup` do aplicativo e `clean_exit` em `RunEvent::Exit`.
- [`src-tauri/src/commands/error_report.rs`](../../../src-tauri/src/commands/error_report.rs): o comando `report_frontend_error` usado pela interface.
- [`src-tauri/src/window/new_tab/bridge.js`](../../../src-tauri/src/window/new_tab/bridge.js): envia erros da página de nova guia pelo caminho autenticado `/error` da ponte.
- [`src/lib/components/status/ErrorReportToast.svelte`](../../../src/lib/components/status/ErrorReportToast.svelte) e [`src/lib/services/error-report.js`](../../../src/lib/services/error-report.js): o cartão e o endereço da issue.
