# Organizador de abas

Abra **Tools → Tabs** ou use a Command Bar. Filtre o workspace atual ou todos os workspaces por título, URL e domínio exato. Grupos de duplicatas comparam URLs completas interpretadas pelo parser, preservando parâmetros de consulta e fragmentos.

Selecione abas elegíveis individualmente ou use **Select extra duplicates**, que preserva a aba ativa ou a usada mais recentemente de cada grupo. Revise os títulos selecionados e confirme antes de fechar. Abas ativas, fixadas, carregando, com formulário alterado, mídia ou download são excluídas. O backend confere novamente essas proteções e informa abas fechadas e preservadas, incluindo abas que desapareceram durante a revisão.

O fechamento em lote persiste uma vez e reconcilia as WebViews nativas uma vez, restaurando o estado se a persistência falhar. Exige a aplicação desktop. Fechar o painel restaura o foco; **Go to tab** fecha Ferramentas e seleciona a aba e seu workspace.
