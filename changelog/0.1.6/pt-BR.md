# Spark 0.1.6

## Novas funcionalidades

- Novos botões para abrir e sair do modo split na lista vertical de abas. O botão no topo da lista abre um split com a aba que você está vendo à esquerda e a última aba usada à direita (uma nova guia em branco, se for a única). O botão na linha entre os painéis sai do split.

## Melhorias

- As extensões instaladas pela Chrome Web Store ou por um arquivo `.crx` agora são verificadas como o Chrome faz: o pacote precisa estar assinado pelo desenvolvedor e, na loja, ser a extensão que você pediu. Um pacote alterado ou errado é recusado, e a versão que você já tinha continua instalada. Arquivos CRX2 antigos não são mais aceitos.
- A seção **Diagnósticos locais** das configurações agora mostra os problemas que o Spark registrou: as últimas entradas do log local de erros (Crítico, Erro, Aviso e Info), da mais nova para a mais antiga, com filtro por nível, botão para copiar e o caminho do arquivo de log. Os contadores internos de desempenho foram para um bloco recolhido, "Desempenho interno".
- As sugestões da barra de endereço vindas do histórico e dos favoritos não mostram mais o endereço da página abaixo do título.
- O estilo de fundo Vidro agora tem dois tipos: **Normal** (o visual translúcido de sempre) e **Vidro líquido** (superfície mais transparente, com desfoque forte, cores realçadas e bordas brilhantes que refletem a luz). Escolha em Configurações → Aparência e layout depois de selecionar Vidro.
- O cabeçalho da lista vertical de abas ficou mais simples: mostra só "Abas (n)" e seus botões (modo split, aba efêmera e nova aba) agora têm o mesmo tamanho e mostram só um ícone; passe o mouse sobre um para ver o nome.
- Com abas verticais e o modo split ativos, agora dá para arrastar uma aba de um lado do split para o outro na lista de abas. Solte em qualquer ponto do outro lado, e a lista mostra onde ela vai ficar enquanto você arrasta. Também dá para soltar sobre a página do outro painel.
- Com abas verticais e o modo split ativos, a lista de abas agora tem uma linha separando as abas de cada lado do split (lado direito e abas fora do split), para você saber quais abas pertencem a cada lado.
- O log de erros agora informa quantas vezes uma entrada idêntica se repetiu e foi ignorada, na próxima entrada gravada para ela.
- O visual da seção Sobre e atualizações nas configurações foi reformulado com um cartão de resumo de status, progresso e avisos mais claros e cartões lado a lado para o repositório do navegador e o mecanismo WebView2.

## Correções

- Atualizado um componente que podia fazer o Spark fechar com um endereço de página feito de propósito para isso.
- As senhas salvas não são mais oferecidas em sites de outras pessoas que só compartilham o final do endereço com o seu, como dois sites diferentes em `github.io` ou `co.uk`. As senhas continuam sendo oferecidas nos subdomínios do seu próprio site.
- Corrigido um problema de segurança em que um site malicioso podia ler o segredo que o Spark usa para falar com os próprios scripts naquela página e usá-lo para pedir seus cartões, dados de contato e senhas salvos ou acionar atalhos do navegador sem você fazer nada. Os scripts de cada função agora têm segredos separados, e as mensagens deles não podem mais ser vistas pela página.
- O log de erros não se enche mais de entradas sobre coisas normais: uma página sem mídia e a aba de configurações que ainda está carregando quando é aberta. Falhas reais continuam sendo registradas.
- O botão **Copiar** em Diagnósticos locais agora funciona mesmo quando o painel não está com o foco do teclado, e avisa quando não foi possível copiar.
- Nas configurações rápidas, a opção selecionada (como o tema claro ou escuro) agora fica claramente destacada nos dois temas. Antes, no tema claro, o botão selecionado parecia mais apagado que o outro.
- As sugestões da barra de endereço voltam a ser legíveis com os estilos de fundo Vidro e Gelo: a lista não deixa mais a página de trás aparecer através das linhas.
- As sugestões da barra de endereço vindas do histórico e dos favoritos agora mostram o ícone de cada site, como nas abas.
- Soltar a aba mostrada em um painel do split sobre a página do outro painel não troca mais os dois painéis de lado. A aba só muda de lado, e o painel de onde ela saiu passa a mostrar outra aba dele.
- O Spark não fecha mais logo ao abrir quando a sessão salva tem uma aba que ele não consegue restaurar (por exemplo, depois de voltar para uma versão mais antiga). Essa aba é ignorada e o problema é registrado no log de erros.
- Erros que impedem o Spark de salvar ou fechar (mostrados no diálogo de fechamento) agora são gravados no log de erros com a falha exata, para poderem ser reportados e corrigidos.
- Um erro dentro de um widget da nova aba (clima, Gmail, Spotify, YouTube) ou do fundo de áudio não deixa mais a página de nova aba quebrada até reiniciar o Spark: os widgets continuam funcionando e o problema é registrado no log de erros. Salvar as configurações dos widgets também não deixa mais os widgets ocupados enquanto as configurações do Spotify e do áudio são aplicadas.
- As mensagens do modo leitura (a linha de autor e o erro "não foi possível extrair"), a dica do botão **Carregar .zip / .crx** das extensões e os rótulos de leitor de tela das notificações agora seguem o idioma da interface. Antes, ficavam sempre em português. No Linux, o modo leitura agora avisa que está disponível apenas no Windows, em vez de mostrar um artigo vazio com uma contagem de palavras inventada.
- O modo leitura agora mostra as imagens do artigo. Antes elas apareciam quebradas, porque a interface do navegador não carrega imagens da web; agora vêm da cópia que a página já carregou, e as imagens que a página não tinha carregado ficam de fora em vez de aparecerem quebradas.
