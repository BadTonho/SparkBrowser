# Spark 0.1.3

## Novas funcionalidades

- O estilo do fundo das janelas agora tem três opções: **Fosco** (sólido), **Vidro** (translúcido e que reflete a luz, com brilho e borda clara) e **Gelo** (um desfoque forte). A antiga escolha "Vidro Fosco" agora se chama **Gelo** e é mantida se você a tinha selecionada.

- Clicar na barra de endereço ou digitar nela agora mostra sugestões logo abaixo: seus sites mais recentes quando ela está vazia e, enquanto você digita, o que o Enter vai fazer, depois favoritos e páginas visitadas que combinam. Use as setas e o Enter, ou clique numa linha.

- Nova opção **Sugestões de pesquisa** (Configurações → Busca e navegação, ligada por padrão): enquanto você digita na barra de endereço, ela pede ao seu mecanismo de busca pesquisas sugeridas e as lista abaixo dos sites que combinam. Funciona com Google, DuckDuckGo, Bing e Brave Search, envia só o que você digitou e nunca é usada em janelas privadas.

- Agora dá para trocar a paleta de cores do navegador em **Configurações → Aparência e disposição**: Grafite (o preto e cinza originais, e o padrão), Oceano, Violeta, Esmeralda, Rosa e Âmbar, cada uma com uma versão para o tema escuro e para o claro. Ela vale para todas as janelas abertas ao mesmo tempo, para a página de nova guia e para o botão do escudo.

- A Barra de Comandos (Ctrl + K) agora abre acima da página, que continua visível, escurecida, atrás dela, em vez de ser trocada por uma tela escura com um cartão de aviso. Ela foi redesenhada para ficar mais limpa e combinar com o resto do navegador (mesma superfície dos outros painéis, linha selecionada discreta, sem brilho), e o campo de texto agora comporta o texto de ajuda.

- Novo botão **Recursos** (desligado por padrão; ligue nos botões da barra das configurações rápidas). Ele abre um painel com a memória que as abas abertas usam e uma página de **Limpeza** onde você escolhe o que limpar: cache, cookies e dados dos sites, histórico de navegação e a lista de downloads, com uma confirmação antes de limpar qualquer coisa.

- O painel **Recursos** agora tem limites e números ao vivo: um limite de memória (acima dele, as abas usadas há mais tempo são hibernadas), um limite do processador (um teto para a parte do processador que as páginas podem usar) e um limite de velocidade da internet, cada um com o uso atual à mostra. Os limites de processador e internet estão completos no Windows e parciais no Linux (só prioridade, e só downloads).

## Melhorias

- No painel de personalização centralizado, as opções **Posição do painel** e **Posição dos widgets de música** agora são pequenas, ficam lado a lado e permanecem na parte de baixo do painel, deixando mais espaço para os cartões.

- Os nomes das opções de personalização da nova guia ficaram mais curtos: **Compacto** e **Centro** para a posição do painel, **Padrão** e **Central** para a posição dos widgets de música.

- A página de nova guia se adapta melhor à largura da janela: os cartões de Clima e Gmail encolhem para caber ao lado da busca, em vez de irem para cima dela, mantendo a busca e os sites recentes à vista em janelas de tamanho médio. Um cartão do Gmail muito alto agora rola dentro dele mesmo.

- Em **Música e vídeo**, Spotify e YouTube agora formam uma lista vertical com os logos, em vez de duas abas lado a lado. No painel centralizado, a lista fica à esquerda e as opções do serviço escolhido à direita. Use as setas para cima e para baixo para alternar entre eles.

- Em **Plano de fundo**, o equalizador de áudio e a mídia de fundo também formam uma lista vertical. No painel centralizado, a lista fica à esquerda e as opções do grupo escolhido à direita, sem precisar de uma rolagem longa.

- O estilo do equalizador de áudio agora é escolhido em uma grade de pequenas prévias, cada uma desenhada na cor que você escolheu, em vez de uma lista suspensa.
- No painel centralizado, a prévia da **Mídia de fundo** ficou bem maior.

- A janela "Fechar o navegador?" ficou com um visual mais limpo: mais espaçamento, a opção "Não me mostrar novamente" mais leve e botões mais claros.

- Em **Relógio**, a fonte, o formato de hora e a posição agora são escolhidos em botões visíveis, em vez de listas suspensas. Cada fonte aparece com um horário de exemplo naquela fonte.

- O menu de configurações do navegador ganhou uma nova ordem: Aparência e disposição, Busca e navegação, Workspaces e tema, Desempenho, Privacidade e segurança, Atalhos de teclado, Sobre e atualizações, Changelog, Diagnósticos locais e Dados e backup.

- Nas configurações do navegador, os seletores de opção ficaram mais fáceis de ler: cada grupo de escolhas (como Pop-up / Guia, Escuro / Claro) é um controle segmentado claro com a opção escolhida preenchida, e os títulos e as descrições ficaram maiores.

- Nas configurações do navegador, toda opção de ligar e desligar agora é o mesmo interruptor, em vez de uma mistura de interruptores e quadrados azuis (por exemplo em Desempenho e em Privacidade e segurança).

- Nas configurações do navegador, o título agora é só **Configurações** e a pesquisa é um único ícone: clique nele, ou em qualquer lugar do espaço livre ao lado, e ele se abre no campo de pesquisa. Ele se fecha de novo quando você o deixa vazio.

- Quando o navegador para de responder, o log de erros agora lista o que ele estava fazendo logo antes (os últimos comandos e painéis) e, no Windows, onde cada parte do app está esperando, para dar para achar a causa em vez de ver só que ele travou.

- Todo comando que falha e todo erro de console da interface agora vão para o log de erros, que segue quatro níveis (Crítico, Erro, Aviso, Informação).

- A barra superior agora mostra por padrão só Configurações, Downloads, Proteção e Extensões. Ligue ou desligue os outros botões (Ferramentas, Barra de favoritos, Comandos, Histórico, Modo Zen, QR Code, WhatsApp) em **Botões da barra**, no menu de configurações rápidas.

- O bloqueador de anúncios agora também esconde os anúncios e caixas de "patrocinado" que as listas de filtros indicam em cada página, e não só bloqueia as requisições. Os anúncios do YouTube são removidos antes de o player recebê-los, e "Auto skip ads" agora também leva ao fim um anúncio que não tem botão de pular.

- Ao arrastar uma aba, agora aparece a própria aba acompanhando o ponteiro (com ícone e título), um espaço tracejado no lugar dela, e as outras abas deslizam suavemente para as novas posições quando você reordena.

- Abrir uma divisão agora é suave: as páginas deslizam para os novos lugares como as abas, e a área onde uma aba arrastada abre uma divisão é bem maior (15% da largura da página, em vez de uma faixa fina na borda).

- A guia de configurações abre mais rápido: não mostra mais uma página branca enquanto carrega (ela fica escura desde o primeiro instante) e a interface percebe mais cedo que a página está pronta.

## Correções

- Corrigido o problema em que escolher uma imagem ou vídeo como plano de fundo da nova guia não tinha efeito no aplicativo instalado.
- Corrigido o aviso "Fechar o navegador?" aparecer por causa da nova guia ou da página de configurações: mudar uma opção ali contava como formulário alterado, e o "Não me mostre novamente" nunca valia.
- Corrigido o estilo do fundo das janelas (Fosco, Vidro, Gelo) não atualizar as janelas na hora; era preciso mexer no controle de opacidade ou de desfoque para ver a mudança.
- Corrigidos os interruptores das configurações do navegador (como a Aceleração de hardware) que pareciam iguais ligados e desligados; a bolinha era branca sobre uma trilha clara. Agora o estado aparece com clareza, nos temas escuro e claro.
- Corrigido o travamento permanente do navegador quando uma nova guia era substituída (por exemplo ao abrir a guia de configurações) com o visualizador de áudio em funcionamento.
- O log de erros de um travamento agora lista primeiro a thread principal e as threads que rodam o código do Spark, para a causa não ser cortada.
