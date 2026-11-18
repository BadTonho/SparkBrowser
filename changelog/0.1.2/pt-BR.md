# Spark 0.1.2

## Novas funcionalidades

- O Spark agora avisa quando algo dá errado: um cartão no canto inferior direito mostra o erro, com **Ver erro** para ler os detalhes e **Enviar erro** para abrir a página de issues do projeto já preenchida. Cobre erros internos, travamentos, erros mostrados pelo navegador, erros de script na interface e na página de nova guia, e uma execução anterior que não fechou normalmente. Nada é enviado sem o seu clique, e endereços de páginas e a sua pasta de usuário são removidos do relato.

- O Spark mantém um registro de erros local (`logs/spark-errors.log` na pasta de dados do Spark) com todos os erros, em quatro níveis (info, aviso, erro e crítico), para você anexar ao reportar um bug. Só erros e erros críticos mostram o cartão; falhas que o Spark antes ignorava em silêncio agora são registradas como avisos.

## Correções

- Corrigido o controle de intensidade visual do plano de fundo de áudio que travava depois de diminuído, impedindo aumentá-lo novamente.
