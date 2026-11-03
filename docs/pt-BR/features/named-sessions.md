# Sessões nomeadas e Ferramentas

Abra **Tools → Sessions**, ou use a Command Bar, para salvar as páginas do workspace atual. As sessões podem ser renomeadas, atualizadas com o workspace atual, excluídas após confirmação e restauradas no workspace atual. A restauração adiciona novos IDs de abas sem fechar ou reutilizar as existentes. Somente a página selecionada elegível é aberta inicialmente; as demais começam hibernadas.

Os snapshots incluem títulos, URLs, ordem, fixação e seleção. Formulários, rolagem e histórico interno das páginas não são capturados. Páginas bloqueadas pelo shield permanecem hibernadas e geram um aviso visível. Migrações SQLite e atualizações de sessão são transacionais; atualizações malsucedidas preservam o conteúdo salvo anterior. A função exige o runtime desktop. Os dados ficam no banco local da aplicação.

Ferramentas abre em um [painel flutuante ancorado](tools-popover.md), mantendo as páginas nativas visíveis, com fechamento por Esc e restauração de foco no botão. A seção Notas oferece [notas por página](page-notes.md).
