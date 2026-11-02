# Abas fechadas recentemente

Use **Ctrl + Shift + T**, a ação da Command Bar ou **Ferramentas → Fechadas recentemente** para reabrir uma página. O atalho pode ser alterado nas Configurações. O aplicativo desktop guarda até 100 fechamentos por 30 dias no SQLite local; o preview no navegador apresenta o recurso como indisponível.

Fechamentos individuais e revisados no organizador salvam URL, título, workspace, posição, fixação e horário na mesma transação da sessão. Hibernação e exclusão de workspace não criam registros. O organizador continua revalidando as proteções antes de fechar abas. Uma falha na transação preserva a sessão anterior e não cria nem consome um registro de fechamento.

A reabertura gera um novo ID, seleciona o workspace original e insere a aba na posição original disponível. Se esse workspace não existir mais, o workspace atual recebe a página e a interface informa a alternativa usada. O registro só é consumido após a persistência. As restrições do Privacy Shield continuam ativas; uma reabertura negada mantém o registro e apresenta o erro.

A lista pode ser apagada com confirmação. Limpar o histórico de navegação também apaga as abas fechadas recentemente. Backups excluem esses registros. A reabertura restaura o endereço e os metadados da página; não restaura formulários, rolagem ou histórico interno de navegação.

Os testes Rust cobrem retenção, posições por workspace e rollback da transação. Execute `npm run test:native -- --case closed-tabs` para fechamentos individuais/em lote, reinício, novos IDs, fixação/posição, atalho real, workspace removido, bloqueio do shield e limpeza do histórico.
