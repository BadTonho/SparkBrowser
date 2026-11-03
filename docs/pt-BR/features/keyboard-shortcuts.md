# Atalhos de teclado entre abas

**Ctrl+Tab** e **Ctrl+Shift+Tab** selecionam a próxima aba ou a anterior, circulando no grupo focado da divisão ou no workspace inteiro fora da divisão. A seleção por número (Ctrl+1–9, com 9 selecionando a última aba), novas abas (Ctrl+T) e a reordenação também usam o grupo focado. Selecionar uma aba na lista vertical unificada dá foco ao seu painel proprietário. Veja [visualização dividida](split-view.md).

Mantenha **Ctrl** pressionado e pressione/solte **W** repetidamente para fechar abas ativas consecutivas. Cada novo pressionamento de W fecha uma aba; segurar W não fecha páginas repetidamente. **Ctrl + Shift + T** reabre abas fechadas recentemente. As combinações continuam configuráveis nas Configurações.

No Windows, os aceleradores nativos das WebViews consultam o estado atual dos modificadores independentemente da fila de mensagens da página anterior. O fechamento por atalho transfere o foco do teclado à página selecionada após reconciliar as WebViews. A transferência é ignorada quando há overlay ou preparação de encerramento, a seleção já mudou ou a janela está minimizada. Fechamentos em segundo plano e reconciliações comuns de layout/resize não solicitam foco.

O cenário nativo Windows `keyboard` envia teclas reais ao executável de teste com identidade verificada, fecha três abas mantendo Ctrl, confere foco e bloqueio de repetição, reabre três abas mantendo Ctrl+Shift e fecha pela interface. O helper verifica o executável em primeiro plano antes de enviar pressionamentos e libera as teclas injetadas em `finally`. Mantenha o desktop de teste disponível durante esse cenário. Nenhum comando do aplicativo ou permissão ACL adicional é criado para testes.
