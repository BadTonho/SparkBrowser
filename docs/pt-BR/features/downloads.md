# Downloads locais retomáveis

O gerenciador abre abaixo do botão da barra e mostra o progresso combinado em sua borda; veja [o painel flutuante](downloads-popover.md).

O gerenciador oferece Pausar, Retomar, Reiniciar, Cancelar e verificação SHA-256 opcional. Transferências por URL e seus arquivos parciais persistem localmente. Reiniciar o aplicativo restaura seu estado sem abrir conexões; a retomada é sempre explícita. Até duas operações executam simultaneamente, com no máximo 64 tarefas não concluídas, incluindo pausadas.

Transferências por URL usam HTTP assíncrono, timeout de conexão de 20 segundos e de inatividade de leitura de 30 segundos, sem limite global enquanto houver progresso. Retomar exige ETag forte, a mesma URL final e `206 Content-Range` correspondente. Validadores fracos/ausentes, conteúdo alterado, intervalos ignorados e respostas inconsistentes exigem reinício explícito. As restrições atuais do shield e de redirecionamentos valem para cada nova requisição.

Os parciais ficam em `.veloce-partials` dentro do diretório de Downloads do sistema. Checkpoints duráveis guardam a quantidade de bytes confirmados e o SHA-256 desse prefixo. Antes de retomar, o Veloce recalcula o hash, rejeita dados danificados/ausentes e trunca qualquer trecho ainda não confirmado. Arquivos concluídos são publicados sem sobrescrever caminhos existentes. O registro de finalização recupera interrupções entre publicar o arquivo e gravar a conclusão; o ID da tarefa impede registros duplicados no histórico. Parciais permanecem até conclusão ou cancelamento/reinício explícito. Limpar o histórico não apaga arquivos nem tarefas pausadas.

Um SHA-256 esperado opcional é conferido antes de concluir uma transferência por URL. Downloads concluídos também podem ser conferidos novamente com um hash esperado. A verificação executa fora do thread da interface; uma divergência nunca apaga um arquivo concluído. Hashes verificam integridade, não fazem varredura contra malware.

Downloads de páginas no Windows usam diretamente operações do WebView2 para progresso, pausa, retomada condicional e cancelamento. Objetos COM permanecem no thread da interface e comandos validam a geração da WebView. Transferências nativas pausadas mantêm sua WebView de origem protegida. Uma operação do motor não pode ser recuperada após reiniciar o aplicativo; tarefas nativas interrompidas oferecem sua página de origem, sem repetir requisições autenticadas, POST ou blob pelo downloader por URL. Arquivos nativos concluídos continuam verificáveis. Linux/macOS mantêm o fluxo do motor sem anunciar esses controles nativos adicionais.

Novos comandos e eventos são exclusivos da interface local. Os registros de transferências usam a migração nomeada `resumable-downloads-v1`, ficam fora dos backups e nunca são sincronizados com um servidor. Os testes redirecionam Downloads somente para o identificador nativo isolado, dentro do diretório de dados do cenário.

Execute `npm run test:native -- --case downloads` para reinício, igualdade dos bytes, intervalos/validadores inválidos, corrupção local, hashes e controles nativos Windows. Os testes Rust cobrem interpretação de intervalos, rollback do registro, conclusão idempotente, validação de caminhos e publicação sem colisões.
