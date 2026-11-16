# Widget "tocando agora" do YouTube

[English](../../en/features/youtube-widget.md)

Abra **Personalizar nova guia → Música e vídeo → YouTube** e ative **Mostrar o que está tocando no YouTube**. O cartão aparece abaixo do Clima e do Spotify, à esquerda. **Música e vídeo** reúne Spotify e YouTube em duas abas; alterne com um clique ou com as setas esquerda e direita.

O cartão é propositalmente minimalista. O cabeçalho é apenas o logotipo do YouTube (o do YouTube Music em music.youtube.com); clicar nele leva à aba do vídeo. Abaixo ficam a miniatura, o título em uma única linha, uma barra de progresso fina com o tempo decorrido e o total, e os controles de anterior, reproduzir/pausar e próximo. Um título que não cabe rola devagar de um lado para o outro, com uma pausa em cada ponta; com movimento reduzido ativado, ele é abreviado com reticências. Transmissões ao vivo não têm duração, então a barra fica oculta nelas. Com várias abas do YouTube abertas, vence a que está tocando, depois a aba ativa e depois a usada mais recentemente. Sem nenhum vídeo do YouTube aberto, o cartão explica como iniciar um.

## Opções de miniatura e pré-visualização

Em **Personalizar nova guia → Música e vídeo → YouTube**:

- **Miniatura:** **Pequena, ao lado do título** (padrão) ou **Cobrir o widget inteiro**, que preenche o cartão com a miniatura atrás de um degradê escuro para que o título, a barra e os controles continuem legíveis.
- **Pré-visualização:** **Só a miniatura** (padrão) ou **Vídeo sem som**, que reproduz o próprio vídeo no lugar da miniatura, ou em todo o cartão no modo de cobertura.

Essas duas escolhas são locais ao dispositivo e compartilhadas por todas as novas guias. Elas ficam fora dos backups.

O vídeo sem som é uma segunda cópia, sem áudio, carregada no player com privacidade aprimorada do YouTube (`youtube-nocookie.com`) dentro do cartão; a aba que você assiste continua com o som. Ele acompanha a aba: começa na posição dela, pausa e retoma junto e salta quando você avança ou volta mais do que alguns segundos. Não é clicável, nunca mostra controles e para enquanto a nova guia está oculta. O player é ampliado para que a barra de título e o logotipo do próprio YouTube fiquem fora do cartão, e só aparece enquanto o vídeo está realmente tocando: ao carregar, ao pausar a aba ou quando o vídeo recusa a incorporação, a miniatura é mostrada no lugar. Usa mais dados e pode mostrar anúncios, e vídeos que proíbem incorporação voltam para a miniatura. É o único caso em que a nova guia contata o YouTube: a política da página permite quadros apenas desse host (`frame-src https://www.youtube-nocookie.com`, também listado no `tauri.conf.json` do app) e continua bloqueando qualquer outra conexão.

## Na barra lateral de guias verticais

Com o layout de guias verticais, o mesmo cartão também aparece na parte de baixo da barra lateral, abaixo da lista de guias, sempre que um vídeo do YouTube ou do YouTube Music estiver aberto. É uma versão compacta: o logotipo (clique nele para ir à aba), a miniatura, o título em uma linha que rola, a barra de progresso com o tempo decorrido e o total, e os controles de anterior, reproduzir/pausar e próximo. Ele segue a mesma escolha de **Miniatura** do cartão da nova guia (pequena ou cobrindo o cartão inteiro), mas nunca reproduz a pré-visualização em vídeo sem som. Some quando nenhum vídeo do YouTube está aberto.

Ele exige que o widget esteja ativado (**Mostrar o que está tocando no YouTube**). Desative **Mostrar também na barra lateral de guias verticais** nas mesmas configurações para manter o cartão só na nova guia. Como a barra lateral está sempre à vista, o Spark pede a posição à aba do vídeo a cada poucos segundos enquanto essa opção está ativa, mesmo sem nova guia visível. Esse pedido é o mesmo relatório de mídia local descrito abaixo e não gera tráfego de rede. A opção é salva com as configurações do widget e, como elas, fica no dispositivo e fora dos backups.

## Como funciona

Não precisa de conta, chave de API ou login. O Spark lê apenas o que já acompanha em cada aba: endereço, título e se está tocando. Uma aba é considerada quando é uma página de vídeo `https` (`/watch?v=…` com um ID de 11 caracteres) em `youtube.com`, `www.youtube.com`, `m.youtube.com` ou `music.youtube.com`. O título perde a contagem de notificações e o sufixo do site.

Os controles agem na própria aba: reproduzir/pausar alterna o elemento de vídeo da página, e próximo e anterior acionam os botões do próprio player. Anterior reinicia o vídeo quando o player não tem botão de anterior. Se a aba foi fechada ou hibernada, o cartão mostra um erro e se atualiza sozinho.

## Atualização e privacidade

O cartão pede à aba do vídeo uma posição atualizada a cada alguns segundos enquanto uma nova guia normal está visível, e cerca de um segundo depois de um controle; entre as amostras a barra avança sozinha enquanto o vídeo toca, então um avanço manual aparece na amostra seguinte. A aba responde pelo mesmo relatório de mídia fixo e verificado por token que já usa para o estado de reprodução, agora com a posição e a duração em segundos; relatórios com números inválidos são ignorados. As atualizações param com o widget desativado ou a página oculta. Nada é armazenado. A opção é local ao dispositivo e fica fora dos backups.

Sem o vídeo sem som, a única requisição de rede é a miniatura pública, `https://i.ytimg.com/vi/<id>/mqdefault.jpg`, buscada pelo Rust com limite de tamanho e verificação da assinatura JPEG, e entregue à página como dados de imagem. Isso preserva a política `connect-src 'none'` da nova guia. Janelas privadas nunca leem abas nem mostram o widget.

## Validação

Testes unitários cobrem a análise de URL e título, hosts, portas, credenciais e IDs rejeitados, os scripts de controle por origem, a renderização, o filtro de miniaturas, o seletor de abas e a ocultação em janelas privadas. A reprodução real e os controles das abas exigem verificação manual com uma página do YouTube.
