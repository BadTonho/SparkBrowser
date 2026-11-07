# Interações de navegação

O modo Zen usa Ctrl+Shift+Z por padrão. Uma configuração salva com o antigo padrão Alt+Z migra quando a nova combinação está livre; outras atribuições são preservadas e um conflito impede essa migração.

O menu nativo de links encaminha o link HTTP(S) selecionado para uma aba em segundo plano no workspace de origem, com seu contêiner e caráter efêmero. Clique do meio e Ctrl+clique usam um gesto confiável sobre um link no documento principal. Pop-ups automáticos continuam sujeitos à configuração de pop-ups. Links dentro de quadros e gestos interceptados pela página precisam de validação nativa específica.

As solicitações de tela cheia da WebView expandem a janela do navegador e a área do conteúdo solicitante. Ao sair da tela cheia, o modo anterior da janela é restaurado; selecionar ou fechar a aba solicitante encerra a tela cheia. Abas em segundo plano não recebem controle sobre a janela.

A área de conteúdo usa os limites internos da moldura para que sua borda não se sobreponha à barra de rolagem vertical da página. Ao alterar essa geometria, teste divisão de tela, redimensionamento e acertos de clique nativos em cada plataforma suportada.

O cenário nativo `navigation-corrections` usa páginas locais e um perfil isolado. Ele passou no Windows para Ctrl+clique, clique do meio, bloqueio de pop-ups automáticos, Escape, persistência da opção de histórico e migração do atalho com outra atribuição personalizada. No Windows, `fullscreen-corrections` passou para Escape e troca de aba, e `new-tab-widgets` passou para configurações, foco, layouts estreitos e divisão de tela. Ainda precisam ser verificados o menu de contexto e a borda da rolagem por testes nativos, outros casos de restauração da tela cheia e a execução com WebKitGTK.
