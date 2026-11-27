# Spark 0.1.6

## Novas funcionalidades

- Novo botão **Baixar vídeo** na barra (desligado por padrão; ligue nas configurações rápidas). Ele lista os vídeos da página carregada, cada um com uma pequena prévia, e salva arquivos de vídeo diretos e streams HLS sem proteção. YouTube, vídeos protegidos (DRM ou criptografados) e transmissões ao vivo não são suportados. A busca tem limite de tempo, então uma página com streams lentos ou pôsteres quebrados não deixa o painel esperando, e um site não consegue usá-la para fazer o Spark enviar pedidos ao seu computador ou à sua rede local.
- Novos botões para abrir e sair do modo split na lista vertical de abas. O botão no topo da lista abre um split com a aba que você está vendo à esquerda e a última aba usada à direita (uma nova guia em branco, se for a única). O botão na linha entre os painéis sai do split. Com abas horizontais, um botão no canto esquerdo da linha do endereço abre e fecha o split, e **Alt+S** faz o mesmo nos dois layouts.
- O campo de pesquisa da página de nova aba agora mostra sugestões, como a barra de endereço: os sites recentes ao clicar nele e, ao digitar, favoritos, histórico e sugestões do mecanismo de busca (que seguem a mesma opção **Sugestões de pesquisa** e nunca são usadas em janela privada).

## Melhorias

- O pop-up de volume ficou mais simples: mostra só a barra de volume e o percentual, sem o título, os avisos e a lista de abas com mídia.
- As extensões instaladas pela Chrome Web Store ou por um arquivo `.crx` agora são verificadas como o Chrome faz: o pacote precisa estar assinado pelo desenvolvedor e, na loja, ser a extensão que você pediu. Um pacote alterado ou errado é recusado, e a versão que você já tinha continua instalada. Arquivos CRX2 antigos não são mais aceitos.
- A seção **Diagnósticos locais** das configurações agora mostra os problemas que o Spark registrou: as últimas entradas do log local de erros (Crítico, Erro, Aviso e Info), da mais nova para a mais antiga, com filtro por nível, botão para copiar e o caminho do arquivo de log. Os contadores internos de desempenho foram para um bloco recolhido, "Desempenho interno".
- As sugestões da barra de endereço vindas do histórico e dos favoritos não mostram mais o endereço da página abaixo do título.
- O estilo de fundo Vidro agora tem dois tipos: **Normal** (o visual translúcido de sempre) e **Vidro líquido** (superfície mais transparente, com desfoque forte, cores realçadas e bordas brilhantes que refletem a luz). Escolha em Configurações → Aparência e layout depois de selecionar Vidro.
- O cabeçalho da lista vertical de abas ficou mais simples: mostra só "Abas (n)" e seus botões (modo split, aba efêmera e nova aba) agora têm o mesmo tamanho e mostram só um ícone; passe o mouse sobre um para ver o nome.
- Com abas verticais e o modo split ativos, agora dá para arrastar uma aba de um lado do split para o outro na lista de abas. Solte em qualquer ponto do outro lado, e a lista mostra onde ela vai ficar enquanto você arrasta. Também dá para soltar sobre a página do outro painel.
- Com abas verticais e o modo split ativos, a lista de abas agora tem uma linha separando as abas de cada lado do split (lado direito e abas fora do split), para você saber quais abas pertencem a cada lado.
- O log de erros agora informa quantas vezes uma entrada idêntica se repetiu e foi ignorada, na próxima entrada gravada para ela.
- O visual da seção Sobre e atualizações nas configurações foi reformulado com um cartão de resumo de status, progresso e avisos mais claros e cartões lado a lado para o repositório do navegador e o mecanismo WebView2.
- As páginas carregam com menos trabalho do escudo de privacidade: a folha de estilo que esconde anúncios com as regras genéricas das listas de filtros agora é preparada uma vez quando as listas carregam, em vez de ser refeita a cada página.
- As sugestões do histórico na barra de endereço e na página de nova aba mostram mais sites diferentes: um site que você visita muito não ocupa mais várias das linhas que a lista lê, e quantas vezes você o visita agora conta por inteiro na ordem da lista.
- O Spark gasta menos energia parado: o analisador do fundo de áudio não acorda mais cinco vezes por segundo quando está desligado ou quando nenhuma nova aba está aberta.
- A janela de configurações abre com muito menos para carregar: cada seção agora é lida na primeira vez que você a abre, em vez de todas de uma vez.
- **Configurações → Aparência e disposição** agora é dividida em categorias (Idioma e janelas, Tema e cores, Fundo da janela, Disposição, Barra e conteúdo), para cada opção ser mais fácil de achar.
- **Configurações → Privacidade e segurança** agora é dividida em categorias (Privacidade de rede, Histórico local, Navegação segura, Escudo de privacidade, Rastreamento), cada opção é um cartão com os mesmos interruptores do resto das configurações, e os sites excluídos são adicionados dentro do cartão da opção e mostrados como etiquetas removíveis.

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
- Quando uma extensão instalada não consegue ser carregada numa nova aba no Windows, o motivo agora é registrado no log de erros, junto com a extensão que falhou. Antes, a falha não deixava rastro.
- Um problema interno do Spark (uma falha em uma parte do navegador) não faz mais outras funções pararem de funcionar em silêncio, ou falharem de novo a cada uso, até reiniciar o Spark. A parte afetada se recupera, continua funcionando, e a recuperação é registrada no log de erros com o lugar onde aconteceu.
- As opções de ligar e desligar de **Barra e conteúdo** nas configurações voltaram a mostrar seus interruptores, em cartões como as outras opções. Antes, apareciam como caixas de seleção simples embaixo do texto.
