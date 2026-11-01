# Notas por página

Abra **Tools → Notes** para editar texto simples da página ativa. Uma nota é compartilhada entre workspaces para cada URL HTTP/HTTPS normalizada. Fragmentos após `#` são ignorados; caminhos e parâmetros de consulta continuam distintos.

Alterações são salvas após 500 ms sem digitação. Rascunhos continuam associados às URLs originais ao trocar de página ou fechar o painel. Uma única fila ordena gravações e exclusões, e respostas atrasadas não substituem texto recente. O estado mostra Unsaved, Saving, Saved ou Error; Retry preserva e reenvia um rascunho com falha. As notas exigem o runtime desktop, ficam no banco local da aplicação e têm limite de 200 KB cada. Encerramento abrupto pode perder alterações ainda não salvas.

Busque no texto salvo, nos títulos e nas URLs; os resultados mostram até 100 correspondências. Atualize a lista após editar para refletir as alterações. Abra a página de uma nota em uma nova aba ou exclua a nota após confirmação. O conteúdo é renderizado como texto, nunca como HTML.
