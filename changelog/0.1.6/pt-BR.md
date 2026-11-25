# Spark 0.1.6

## Novas funcionalidades

- Novos botões para abrir e sair do modo split na lista vertical de abas. O botão no topo da lista abre um split com a aba que você está vendo à esquerda e a última aba usada à direita (uma nova guia em branco, se for a única). O botão na linha entre os painéis sai do split.

## Melhorias

- Com abas verticais e o modo split ativos, agora dá para arrastar uma aba de um lado do split para o outro na lista de abas. Solte em qualquer ponto do outro lado, e a lista mostra onde ela vai ficar enquanto você arrasta. Também dá para soltar sobre a página do outro painel.
- Com abas verticais e o modo split ativos, a lista de abas agora tem uma linha separando as abas de cada lado do split (lado direito e abas fora do split), para você saber quais abas pertencem a cada lado.
- O log de erros agora informa quantas vezes uma entrada idêntica se repetiu e foi ignorada, na próxima entrada gravada para ela.
- O visual da seção Sobre e atualizações nas configurações foi reformulado com um cartão de resumo de status, progresso e avisos mais claros e cartões lado a lado para o repositório do navegador e o mecanismo WebView2.

## Correções

- Soltar a aba mostrada em um painel do split sobre a página do outro painel não troca mais os dois painéis de lado. A aba só muda de lado, e o painel de onde ela saiu passa a mostrar outra aba dele.
- O Spark não fecha mais logo ao abrir quando a sessão salva tem uma aba que ele não consegue restaurar (por exemplo, depois de voltar para uma versão mais antiga). Essa aba é ignorada e o problema é registrado no log de erros.
- Erros que impedem o Spark de salvar ou fechar (mostrados no diálogo de fechamento) agora são gravados no log de erros com a falha exata, para poderem ser reportados e corrigidos.
