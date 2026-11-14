# Encerramento protegido

Fechar a janela, Alt+F4 e solicitações normais de saída mantêm a interface aberta enquanto as notas pendentes são salvas. Uma revisão traduzida lista formulários alterados, mídia e transferências de todos os workspaces. **Continuar navegando** recebe o foco inicial. Cancelar a revisão mantém as transferências em execução. Mudanças na atividade exigem outra confirmação.

Após **Sair mesmo assim**, transferências por URL pausam e persistem seus checkpoints. Downloads nativos perdem a conexão com a WebView; a revisão avisa que não poderão ser retomados ao reabrir. O gravador de navegação deixa de aceitar tarefas e esvazia a fila antes do bloqueio final de abas. Abas da sessão, divisões e marcador de saída limpa são confirmados juntos. Falhas de persistência mantêm o navegador aberto e os rascunhos disponíveis para nova tentativa.

A revisão lista cada página com seu host e uma etiqueta do que está em risco (formulário alterado, download de página, mídia em reprodução). **Não me mostre novamente ao fechar o navegador**, nesse diálogo, ou **Pular a confirmação ao fechar**, em Configurações > Desempenho, grava uma preferência local (`spark_skip_exit_review`). Ela só vale para fechar o navegador, e só quando nada seria perdido de vez: mídia e transferências que podem ser retomadas. Formulários alterados e downloads de páginas que não retomam continuam mostrando a revisão, e instalar uma atualização ou reiniciar o motor sempre a mostram. A preferência é gravada ao escolher **Sair mesmo assim** e pode ser desligada de novo em Configurações.

O atualizador usa o mesmo coordenador e a mesma sequência de persistência. Transferências ativas precisam terminar ou ser canceladas antes da instalação. Se o instalador retornar, o marcador de execução e o gravador de navegação voltam ao estado em execução. O preview não encerra processos desktop.

Execute `node tools/testing/native-smoke.mjs --case shutdown` com a build isolada para validar debounce de notas, falha de gravação e nova tentativa, foco padrão seguro, mudanças de atividade e reinício limpo. O cenário não executa instaladores.
