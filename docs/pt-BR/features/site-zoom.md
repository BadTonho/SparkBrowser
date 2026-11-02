# Zoom persistente por site

A barra de endereço mostra o zoom da página focada e oferece controles de diminuir, restaurar e aumentar. Os atalhos padrão são **Ctrl + +**, **Ctrl + -** e **Ctrl + 0**, incluindo equivalentes do teclado numérico; altere as combinações nas Configurações. Os níveis variam de 25% a 500%, com 100% como padrão.

O zoom pertence à origem HTTP/HTTPS: protocolo, domínio e porta. Workspaces compartilham a preferência. A alteração atualiza todas as páginas residentes dessa origem sem recriar páginas nem transferir foco. Abas hibernadas recebem o valor ao despertar. Na divisão, os controles acompanham o painel focado. Configurações → Busca e Navegação lista origens salvas com restauração individual. O preview informa que o zoom exige o aplicativo desktop.

No Windows, os observadores descartam gerações antigas de WebViews e callbacks de navegação. A navegação aplica o valor salvo, sem persistir os padrões automáticos do motor. Os atalhos configurados usam a ponte do navegador; os atalhos de zoom do motor ficam desabilitados para evitar ações duplicadas. Consulte a [referência do controlador WebView2](https://learn.microsoft.com/en-us/microsoft-edge/webview2/reference/win32/icorewebview2controller).

A exportação de backup usa a versão 3 e inclui preferências de zoom. As versões 1 e 2 continuam importáveis. O zoom só é aplicado pela tarefa persistente de preferências quando **Aplicar preferências do backup** estiver marcado. Origens locais existentes prevalecem nos conflitos; repetir a tarefa não as sobrescreve nem importa dados novamente. Transferências, parciais e marcadores de execução continuam excluídos.

Validação: `node tools/testing/native-smoke.mjs --case zoom`, além do cenário existente `backup`, com a build nativa isolada.
