# Spark 0.1.6

## Melhorias

- Com abas verticais e o modo split ativos, agora dá para arrastar uma aba de um lado do split para o outro na lista de abas. Solte sobre uma aba ou sobre o separador do outro lado.
- Com abas verticais e o modo split ativos, a lista de abas agora tem um separador para cada lado do split (painel esquerdo, painel direito e abas fora do split), para você saber quais abas pertencem a cada lado.
- O log de erros agora informa quantas vezes uma entrada idêntica se repetiu e foi ignorada, na próxima entrada gravada para ela.
- O visual da seção Sobre e atualizações nas configurações foi reformulado com um cartão de resumo de status, progresso e avisos mais claros e cartões lado a lado para o repositório do navegador e o mecanismo WebView2.

## Correções

- O Spark não fecha mais logo ao abrir quando a sessão salva tem uma aba que ele não consegue restaurar (por exemplo, depois de voltar para uma versão mais antiga). Essa aba é ignorada e o problema é registrado no log de erros.
- Erros que impedem o Spark de salvar ou fechar (mostrados no diálogo de fechamento) agora são gravados no log de erros com a falha exata, para poderem ser reportados e corrigidos.
