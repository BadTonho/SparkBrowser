# Spark 0.1.4

## Novas funcionalidades

- **Modo somente HTTPS** (Configurações → Privacidade e segurança → Privacidade de rede): abre a versão segura de todos os sites. Se ela não puder ser acessada, um aviso pergunta antes de continuar por HTTP, uma vez e só para aquele endereço. Endereços locais, como localhost, não são alterados. Vem desligado.
- **DNS sobre HTTPS** (no mesmo lugar): procura os nomes dos sites pela Cloudflare ou pelo Google para que a rede não possa lê-los nem alterá-los, sem voltar ao DNS comum. Vem desligado e é aplicado depois de reiniciar. Chamadas de vídeo (WebRTC) e as requisições de atualização e de ícones do próprio navegador não são cobertas.
- **A mídia de fundo agora pode ser reposicionada**: arraste a pré-visualização em Configurações → Plano de fundo → Mídia de fundo para escolher qual parte da imagem ou do vídeo fica visível na página de nova guia.
- A **mídia de fundo agora pode receber zoom**: um novo controle de Zoom (25% a 300%, então também dá para diminuir a imagem) em Configurações → Plano de fundo → Mídia de fundo muda o quanto da imagem ou do vídeo aparece: 100% preenche a tela, menos mostra mais dela (até aparecer inteira, com o fundo ao redor), mais amplia a partir do centro. A pré-visualização agora tem o formato da sua tela, então mostra o enquadramento real.

## Melhorias

- A entrada **Sessão Tor** não aparece mais nas configurações de privacidade: ela foi adiada até que vazamentos de tráfego possam ser descartados. Nada dela esteve ativo.
- A página **Sobre e atualizações** agora mostra o link do repositório do navegador no GitHub, com um botão que o abre em uma nova guia.
- A **intensidade do equalizador de áudio** agora muda o tamanho do próprio efeito (barras, anéis, orbes, partículas e os demais crescem ou encolhem), e não só o quanto ele aparece. A faixa passou de 0,5–2 para 0,25–4 e o efeito acompanha o controle enquanto você arrasta. Um novo controle de **Opacidade do efeito**, ao lado, define o quanto o efeito é transparente. Os dois controles agora mostram o valor em porcentagem (na intensidade, 100% é onde a barra terminava antes e agora ela vai até 200%).

## Correções

- Corrigida a opção **Mostrar cantos arredondados no conteúdo**: com ela desligada, ainda sobrava uma borda (um espaço de 8px) ao redor da página. Agora a página ocupa toda a área quando está desligada.
- Corrigida a guia de configurações (e a página de nova guia) aparecendo com o tema trocado quando o **Modo escuro nos sites** estava ligado: o navegador escurecia as próprias páginas, então o tema escuro parecia claro e o claro parecia escuro. As páginas do próprio navegador agora nunca são alteradas por essa opção.
- Corrigidas as sugestões da barra de endereço que continuavam na tela ao trocar de guia ou clicar em outro ponto da barra de ferramentas. Agora a lista fecha.
- Corrigidos os atalhos de teclado (F5 e os demais) que paravam de funcionar depois de clicar numa área vazia da barra de guias: o foco do teclado se perdia. Agora os atalhos continuam funcionando.
- Corrigido o F5 (e o Ctrl+Shift+R) recarregando a própria interface do navegador, o que fazia as guias piscarem, em vez de recarregar a página. Agora eles recarregam a página ativa.
- Corrigido o diálogo "Fechar o navegador?" aparecendo sem nada para salvar: o backup feito ao fechar agora só é montado quando algo mudou desde o último backup, então fechar sem novidades é instantâneo.
- Corrigidos os botões laterais do mouse (4 e 5) que voltavam ou avançavam mais de uma página: o navegador também navegava sozinho ao soltar o botão. Agora eles movem exatamente uma página.
- Corrigido o cartão do YouTube na nova guia: não dava para clicar nem arrastar a barra de progresso. Agora ela pula para o momento em que você clica ou arrasta, na guia que está tocando.
- Corrigido o cartão do YouTube na nova guia que não fazia nada quando a guia controlada estava hibernada. Agora, ao apertar play (ou qualquer controle), essa guia acorda em segundo plano, recarrega a página e inicia o vídeo, sem sair da nova guia.
