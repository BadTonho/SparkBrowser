# Permissões por site no Windows

Clique no indicador **Site permissions** da barra de endereço para controlar câmera, microfone, localização e notificações da origem da página focada. **Settings → Performance & Privacy → Site permissions** lista decisões lembradas. Origens incluem protocolo, host e porta; regras são compartilhadas entre workspaces. Ask remove uma regra salva, Allow permite solicitações futuras e Block as nega. Regras ficam no perfil nativo do WebView2, sem um segundo banco.

O painel do site atual e os pedidos de permissão usam um WebView nativo de sobreposição acima do conteúdo da página. O cartão é opaco, enquanto o restante da sobreposição é transparente, mantendo a página visível. As ações são encaminhadas à interface local por uma ponte interna vinculada à sessão; a sobreposição não recebe permissões IPC da aplicação.

Para pedidos de iframe, o diálogo também mostra a origem nativa do frame. O WebView2 pode aplicar uma permissão de mídia à origem que incorpora o conteúdo; quando as origens diferem, o diálogo identifica explicitamente a origem que recebe a decisão lembrada. Navegação e destruição do frame cancelam seus pedidos pendentes.

Novas solicitações mostram um diálogo do navegador com a origem solicitante e, para conteúdo incorporado, a origem da página que o contém. Escolha Allow once, Always allow ou Block; a caixa de seleção lembra o bloqueio. Escape bloqueia somente o pedido atual. Solicitações são processadas uma por vez, com até 16 pendentes; excedentes são negadas. Navegação, fechamento/hibernação da view ou timeout de 60 segundos cancelam pedidos pendentes. IDs antigos não podem conceder permissões a documentos ou views substitutos. Outros tipos de permissão mantêm o comportamento do motor.

Handlers são instalados antes da navegação externa. Objetos COM dos pedidos permanecem no thread da interface; somente metadados chegam à interface local. Decisões temporárias não são salvas no perfil. Alterar uma política afeta novas solicitações e não interrompe captura de mídia existente; o painel do site oferece recarregar e pede confirmação quando há formulários/mídia/downloads detectados. Permissões do sistema operacional e disponibilidade de dispositivos continuam valendo.

Controles ficam indisponíveis em outras plataformas, no preview ou quando as APIs necessárias do WebView2 não existem. Testes nativos usam identificador de aplicação isolado, páginas controladas em loopback e a opção de dispositivo de mídia simulado do Chromium. Veja [testes nativos](../contributing/native-tests.md).
