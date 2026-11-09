# Leitor de feeds

O painel Ferramentas oferece assinaturas RSS e Atom armazenadas localmente. Cadastre um nome e endereço HTTP(S), escolha um intervalo entre 15 e 1.440 minutos (60 por padrão) e atualize manualmente ou aguarde a atualização periódica enquanto o aplicativo estiver aberto. Cada feed mantém até 200 itens. Os últimos itens válidos permanecem disponíveis quando uma requisição ou análise XML falha. Alterar o endereço do feed remove os itens da origem anterior; excluir uma assinatura remove seus itens locais.

As respostas dos feeds são limitadas a 2 MiB. DTDs e URLs de itens que não sejam HTTP(S) são rejeitados. O texto dos artigos é exibido como texto, sem executar HTML, e o leitor não carrega imagens ou recursos incorporados remotos. Abrir um item usa a navegação normal do navegador. Assinaturas e itens de feeds não fazem parte do formato atual de backup.

Uma flag experimental opcional em Diagnóstico local oculta as prévias dos artigos na lista. Ela começa desligada, é persistida localmente e passa a valer ao reabrir o painel. Restaurar experimentos a desativa.

Em 7 de novembro de 2026, um teste nativo isolado passou no Windows para cadastro, atualização, retenção após XML inválido, persistência após reinício e redefinição da flag experimental. A validação da interface nativa e do Linux permanece pendente. Os demais itens da etapa 10, incluindo grupos de abas, abertura automática de downloads e atalhos do sistema operacional, não são implementados por este recurso.

Títulos e links RSS aceitam CDATA. O texto aninhado dos artigos preserva espaços, e os links Atom selecionam alternativas de leitura em vez de recursos self/API/anexos. As atualizações compartilham um cliente HTTP, usam no máximo quatro requisições simultâneas e rejeitam atualizações duplicadas de uma assinatura já em andamento. Atualizações manuais ocupadas podem ser repetidas; atualizações agendadas que falhem ou sejam adiadas continuam elegíveis em um ciclo posterior.
