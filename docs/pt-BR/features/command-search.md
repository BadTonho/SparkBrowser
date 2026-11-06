# Notas e sessões na Command Bar

A digitação pesquisa notas salvas por título, URL e texto e sessões por nome e títulos/URLs das páginas após 180 ms sem digitar. A correspondência ignora maiúsculas e acentos e trata caracteres especiais literalmente. Os resultados mostram prévias curtas como texto e contagens, com identificações explícitas para notas e sessões. A busca não transporta documentos completos de notas ou sessões.

As abas abertas são pesquisadas em todos os workspaces. Cada resultado de aba identifica seu workspace; ao selecioná-lo pelo teclado, o navegador ativa a aba e seu workspace. A busca usa o atalho configurável existente da Command Bar.

Cada categoria retorna até duas correspondências. A contagem descreve as correspondências exibidas; `2+` indica que existem outros resultados, sem calcular o total exato. Índices transacionais de trigramas atendem consultas com três ou mais caracteres. Consultas menores e consultas contendo NUL percorrem campos normalizados e mantêm o mesmo comportamento de busca literal por trechos.

A lista contém até doze itens: no máximo três abas e dois de cada outra categoria. Uma consulta preenchida reserva duas ações de navegação. Respostas antigas não substituem os resultados de consultas recentes. A seleção por teclado ou ponteiro continua associada ao item enquanto chegam resultados assíncronos.

Selecionar uma nota abre seu editor em Ferramentas, associado à URL original e preservando qualquer rascunho. **Abrir página** é uma ação separada; **Acompanhar a página ativa** volta a acompanhar a navegação. O conteúdo continua como texto simples. Selecionar uma sessão abre sua lista de páginas com indicação de fixação. **Restaurar** adiciona abas explicitamente ao workspace atual. Itens removidos são rejeitados; uma sessão alterada precisa ser revisada novamente antes da restauração.

Consultas e dados permanecem locais. Os comandos têm acesso restrito à interface local `main`. O preview não retorna resultados de busca persistente de notas ou sessões. Os testes cobrem buscas Unicode limitadas, respostas atrasadas, limites por categoria, URLs originais das notas, itens removidos e alterações de sessões revisadas. Execute `node tools/testing/native-smoke.mjs --case command-search` com a build nativa isolada.


## Calculadora e conversão de unidades

Expressões aritméticas completas com `+`, `-`, `*`, `/`, sinais unários e parênteses são calculadas localmente com a precedência habitual. Uma expressão reconhecida mostra apenas o resultado; Enter ou clique copia o valor formatado e não navega. O separador decimal e a formatação seguem o idioma da interface. Conversões aceitam `valor unidade to unidade` ou `valor unidade em unidade` nas unidades iniciais de comprimento, massa, temperatura e volume. Unidades desconhecidas ou incompatíveis, divisão por zero e valores fora do intervalo exibem uma explicação traduzida. Funções, potências e porcentagem não são aceitas.

As unidades iniciais são comprimento: `mm`, `cm`, `m`, `km`, `in`, `ft`, `yd`, `mi`; massa: `mg`, `g`, `kg`, `oz`, `lb`; temperatura: `°C`, `°F`, `K`; volume: `mL`, `L`, `US fl oz`, `US cup`, `US gal`. O cenário nativo da Command Bar verifica a prioridade do resultado e a cópia por Enter e clique sem navegar.
