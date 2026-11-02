# Notas e sessões na Command Bar

A digitação pesquisa notas salvas por título, URL e texto e sessões por nome e títulos/URLs das páginas após 180 ms sem digitar. A correspondência ignora maiúsculas e acentos e trata caracteres especiais literalmente. Os resultados mostram prévias curtas como texto e contagens, com identificações explícitas para notas e sessões. A busca não transporta documentos completos de notas ou sessões.

A lista contém até doze itens: no máximo três abas e dois de cada outra categoria. Uma consulta preenchida reserva duas ações de navegação. Respostas antigas não substituem os resultados de consultas recentes. A seleção por teclado ou ponteiro continua associada ao item enquanto chegam resultados assíncronos.

Selecionar uma nota abre seu editor em Ferramentas, associado à URL original e preservando qualquer rascunho. **Abrir página** é uma ação separada; **Acompanhar a página ativa** volta a acompanhar a navegação. O conteúdo continua como texto simples. Selecionar uma sessão abre sua lista de páginas com indicação de fixação. **Restaurar** adiciona abas explicitamente ao workspace atual. Itens removidos são rejeitados; uma sessão alterada precisa ser revisada novamente antes da restauração.

Consultas e dados permanecem locais. Os comandos têm acesso restrito à interface local `main`. O preview não retorna resultados de busca persistente de notas ou sessões. Os testes cobrem buscas Unicode limitadas, respostas atrasadas, limites por categoria, URLs originais das notas, itens removidos e alterações de sessões revisadas. Execute `node tools/testing/native-smoke.mjs --case command-search` com a build nativa isolada.
