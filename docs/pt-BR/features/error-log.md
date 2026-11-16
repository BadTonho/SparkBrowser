# Registro de erros e notificações de erro

[English](../../en/features/error-log.md)

O Spark avisa o usuário quando algo dá errado e mantém um registro local, para que bugs possam ser reportados já com os detalhes coletados. O Spark nunca envia nada sozinho: quem escolhe enviar o erro é o usuário.

## Notificação

Quando um erro acontece, um cartão aparece no **canto inferior direito** da janela do navegador (até três por vez, os demais esperam). Ele fica até ser dispensado e tem dois botões:

- **Ver erro** mostra os detalhes dentro do cartão.
- **Enviar erro** abre uma nova aba do Spark na página de **nova issue** do projeto (`https://github.com/BadTonho/SparkBrowser/issues/new`) com título e texto já preenchidos: versão do Spark, sistema, horário, tipo, resumo e detalhes (cortados para caber no limite do endereço). O usuário revisa e envia no GitHub. Se a página não puder ser aberta, o cartão avisa.

O mesmo erro aparece no máximo uma vez por minuto e no máximo cinco cartões aparecem por minuto, então um laço não consegue inundar a tela.

## O que conta como erro

| Tipo | Origem |
| --- | --- |
| `panic` | Um erro interno inesperado no código Rust (em qualquer thread) |
| `hang` | A thread principal não respondeu a um vigia por 10 segundos; é reportado quando ela volta a responder, com a duração |
| `native` | Qualquer erro que o Spark já mostrava pelo evento `browser-error` (downloads, privacidade, operações de abas etc.). Bloqueios do escudo ("Navigation blocked") são intencionais e não são reportados |
| `interface` | Erro de script ou rejeição não tratada na interface do navegador (shell, configurações, ferramentas, downloads) |
| `new-tab` | Erro de script ou rejeição não tratada na página de nova guia (até 3 por carregamento) |
| `unclean-exit` | A execução anterior deixou o marcador `session.running`: uma queda, um travamento fechado à força ou falta de energia. Aparece alguns segundos depois de iniciar |

Erros que o Spark trata em silêncio sem nunca levantar um evento de erro (um `let _ = ...` em Rust) não são reportados.

## Arquivo de registro

`logs/spark-errors.log` dentro da pasta de dados do Spark (no Windows, `%APPDATA%\org.spark.browser\logs`). Cada relato, cada início e cada fechamento normal são acrescentados ali. Em 512 KB ele é renomeado para `spark-errors.log.old`, então nunca usa mais de cerca de 1 MB. Janelas privadas não escrevem nele nem mostram esses cartões.

## Privacidade

Os relatos são redigidos antes de serem registrados ou mostrados: a pasta do usuário vira `~` e qualquer endereço `http(s)://` é cortado depois do host (`https://example.test/…`). Resumos são limitados a 300 caracteres e detalhes a 6.000.

## Código

- [`src-tauri/src/error_log.rs`](../../../src-tauri/src/error_log.rs): registro, redação, limite de frequência, panic hook, vigia de travamento e marcador de execução. `install` roda em `run()` antes de qualquer janela abrir, `watch` no `setup` do aplicativo e `clean_exit` em `RunEvent::Exit`.
- [`src-tauri/src/commands/error_report.rs`](../../../src-tauri/src/commands/error_report.rs): o comando `report_frontend_error` usado pela interface.
- [`src-tauri/src/window/new_tab/bridge.js`](../../../src-tauri/src/window/new_tab/bridge.js): envia erros da página de nova guia pelo caminho autenticado `/error` da ponte.
- [`src/lib/components/status/ErrorReportToast.svelte`](../../../src/lib/components/status/ErrorReportToast.svelte) e [`src/lib/services/error-report.js`](../../../src/lib/services/error-report.js): o cartão e o endereço da issue.
