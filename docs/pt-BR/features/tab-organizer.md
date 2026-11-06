# Organizador de abas

Abra **Tools → Tabs** ou use a Command Bar. Filtre o workspace atual ou todos os workspaces por título, URL e domínio exato. Grupos de duplicatas comparam URLs completas interpretadas pelo parser, preservando parâmetros de consulta e fragmentos.

Selecione abas elegíveis individualmente ou use **Select extra duplicates**, que preserva a aba ativa ou a usada mais recentemente de cada grupo. Revise os títulos selecionados e confirme antes de fechar. Abas ativas, fixadas, carregando, com formulário alterado, mídia ou download são excluídas. O backend confere novamente essas proteções e informa abas fechadas e preservadas, incluindo abas que desapareceram durante a revisão.

O fechamento em lote persiste uma vez e reconcilia as WebViews nativas uma vez, restaurando o estado se a persistência falhar. Exige a aplicação desktop. Fechar o painel restaura o foco; **Go to tab** fecha Ferramentas e seleciona a aba e seu workspace.

As listas vertical e horizontal também permitem seleção múltipla no workspace atual: Ctrl/Meta-clique alterna a seleção de uma aba e Shift-clique seleciona o intervalo inclusivo a partir da âncora. Quem usa o teclado pode combinar Ctrl/Meta ou Shift com Enter ou Espaço. A barra de seleção permite revisar as abas antes de fechá-las; abas protegidas são identificadas e o backend verifica novamente seu estado antes de aplicar a ação. A aba ativa é rolada para dentro da área visível quando muda.
