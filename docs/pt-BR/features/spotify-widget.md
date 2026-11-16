# Widget do Spotify

[English](../../en/features/spotify-widget.md)

Abra **Personalizar nova guia → Música e vídeo → Spotify**, ative **Mostrar Spotify** e conecte uma conta do Spotify. O cartão aparece abaixo do Clima à esquerda, ou no início dessa coluna quando o Clima está desativado. Janelas estreitas empilham a coluna esquerda antes do Gmail e da pesquisa.

O cartão é propositalmente minimalista e tem o mesmo visual do [widget do YouTube](youtube-widget.md). O cabeçalho é apenas o logotipo do Spotify; clicar nele abre o Web Player no painel atual. Abaixo ficam a capa, o título em uma única linha, o artista ou podcast, uma barra de progresso fina com o tempo decorrido e o total, e os controles de anterior, reproduzir/pausar e próxima do dispositivo ativo da conta. Um título que não cabe rola devagar de um lado para o outro, com uma pausa em cada ponta; com movimento reduzido ativado, ele é abreviado com reticências. O nome do dispositivo ativo aparece ao passar o mouse sobre o título. Abra o Spotify no computador ou celular e inicie a reprodução primeiro.

Em **Miniatura**, nas configurações do Spotify, escolha **Pequena, ao lado do título** (padrão) ou **Cobrir o widget inteiro**, que preenche o cartão com a capa atrás de um degradê escuro. A escolha é local ao dispositivo e fica fora dos backups. A API do Spotify não oferece vídeo, então não há opção de pré-visualização em vídeo aqui. O widget não transmite áudio, escolhe dispositivos, muda o volume, pesquisa faixas ou edita playlists. O Spotify pode restringir controles em determinados dispositivos ou conteúdos.

## Conectar sua conta

1. Use uma conta Spotify Premium e crie um aplicativo no [Spotify Developer Dashboard](https://developer.spotify.com/dashboard), selecionando Web API.
2. Cadastre `http://127.0.0.1/callback` como URI de redirecionamento, sem porta. O Spotify permite portas dinâmicas para endereços IP de loopback; `localhost` não é aceito. Consulte os [requisitos de redirecionamento](https://developer.spotify.com/documentation/web-api/concepts/redirect_uri).
3. Adicione sua conta aos usuários permitidos do aplicativo quando necessário. O modo de desenvolvimento permite atualmente até cinco usuários autenticados e exige Premium do proprietário do aplicativo. Consulte os [modos de cota](https://developer.spotify.com/documentation/web-api/concepts/quota-modes).
4. Cole o **Client ID** do aplicativo nas configurações do Spotify, selecione **Salvar Client ID** e depois **Conectar Spotify**. Não é necessário client secret.
5. Conclua o consentimento no navegador externo e volte ao Spark. Os escopos solicitados são `user-read-playback-state` e `user-modify-playback-state`. A autorização usa PKCE e um listener temporário em loopback com validação de state; ela pode ser cancelada ou expirar após dois minutos.

No Windows, a autorização abre o Microsoft Edge e os refresh tokens usam o Gerenciador de Credenciais do Windows. No Linux, instale `secret-tool` (pacote de ferramentas libsecret da distribuição), execute um Secret Service desbloqueado, como GNOME Keyring ou o serviço compatível do KWallet, e instale um navegador externo disponível no PATH: Firefox, Chromium, Chrome ou Edge. A ausência de navegador ou armazenamento seguro gera um erro; tokens nunca são salvos em arquivos de texto simples.

## Atualização, desconexão e privacidade

A reprodução é consultada a cada dez segundos enquanto houver uma nova guia normal visível, com consultas compartilhadas entre guias e workspaces. Os controles provocam uma atualização imediata. As atualizações param quando o widget está desativado ou oculto. O progresso reflete a última amostra do servidor. Falhas preservam a última exibição e desativam temporariamente os controles; limites de consulta respeitam `Retry-After`. Autorizações expiradas solicitam reconexão. Erros de acesso negado podem indicar ausência de Premium, conta fora dos usuários permitidos ou restrições do dispositivo.

Desativar o widget preserva a conexão. **Desconectar** limpa os dados locais da conta, remove a credencial e registra um marcador persistente de desconexão para impedir restauração em caso de falha na exclusão. Para revogar também o consentimento no Spotify, remova o aplicativo configurado para o Spark na página de aplicativos da sua conta Spotify. Alterar o Client ID exige reconectar.

Preferências e Client ID são locais ao dispositivo e excluídos dos backups. Refresh tokens são isolados por aplicativo/perfil e provedor; access tokens e dados de reprodução ficam na memória. As credenciais nunca entram no documento da interface. O Rust busca capas com tamanho limitado somente no servidor de imagens do Spotify e as envia como dados de imagem, preservando a política `connect-src 'none'` da nova guia. Janelas privadas não conectam, consultam ou mostram dados da conta Spotify.

Os testes automatizados usam fixtures artificiais de OAuth e reprodução; nunca abrem consentimento real ou usam o cofre de credenciais do sistema. O consentimento real e a integração com o cofre exigem verificação manual no sistema operacional correspondente.
