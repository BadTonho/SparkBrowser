# Spark 0.1.4

## Novas funcionalidades

- **Modo somente HTTPS** (Configurações → Privacidade e segurança → Privacidade de rede): abre a versão segura de todos os sites. Se ela não puder ser acessada, um aviso pergunta antes de continuar por HTTP, uma vez e só para aquele endereço. Endereços locais, como localhost, não são alterados. Vem desligado.
- **DNS sobre HTTPS** (no mesmo lugar): procura os nomes dos sites pela Cloudflare ou pelo Google para que a rede não possa lê-los nem alterá-los, sem voltar ao DNS comum. Vem desligado e é aplicado depois de reiniciar. Chamadas de vídeo (WebRTC) e as requisições de atualização e de ícones do próprio navegador não são cobertas.
- **A mídia de fundo agora pode ser reposicionada**: arraste a pré-visualização em Configurações → Plano de fundo → Mídia de fundo para escolher qual parte da imagem ou do vídeo fica visível na página de nova guia.

## Melhorias

- A entrada **Sessão Tor** não aparece mais nas configurações de privacidade: ela foi adiada até que vazamentos de tráfego possam ser descartados. Nada dela esteve ativo.

## Correções

- Corrigida a opção **Mostrar cantos arredondados no conteúdo**: com ela desligada, ainda sobrava uma borda (um espaço de 8px) ao redor da página. Agora a página ocupa toda a área quando está desligada.
- Corrigida a guia de configurações (e a página de nova guia) aparecendo com o tema trocado quando o **Modo escuro nos sites** estava ligado: o navegador escurecia as próprias páginas, então o tema escuro parecia claro e o claro parecia escuro. As páginas do próprio navegador agora nunca são alteradas por essa opção.
- Corrigidas as sugestões da barra de endereço que continuavam na tela ao trocar de guia ou clicar em outro ponto da barra de ferramentas. Agora a lista fecha.
- Corrigidos os atalhos de teclado (F5 e os demais) que paravam de funcionar depois de clicar numa área vazia da barra de guias: o foco do teclado se perdia. Agora os atalhos continuam funcionando.
- Corrigido o F5 (e o Ctrl+Shift+R) recarregando a própria interface do navegador, o que fazia as guias piscarem, em vez de recarregar a página. Agora eles recarregam a página ativa.
- Corrigido o diálogo "Fechar o navegador?" aparecendo sem nada para salvar: o backup feito ao fechar agora só é montado quando algo mudou desde o último backup, então fechar sem novidades é instantâneo.
- Corrigidos os botões laterais do mouse (4 e 5) que voltavam ou avançavam mais de uma página: o navegador também navegava sozinho ao soltar o botão. Agora eles movem exatamente uma página.
