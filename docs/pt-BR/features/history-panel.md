# Painel de histórico

[English](../../en/features/history-panel.md)

O painel **Histórico** (botão da barra ou atalho) lista as páginas que você visitou, das mais novas para as mais antigas, sobre o lado esquerdo da janela.

## Layout

- **Cabeçalho:** o título, um campo de pesquisa arredondado (pesquisa títulos e endereços) e **Limpar tudo**.
- **Um cartão por dia**, com o título "Hoje - sexta-feira, 27 de novembro de 2026", "Ontem - …" ou só a data, no idioma da interface.
- **Uma linha por visita:** uma caixa de seleção, o horário, o ícone do site (a primeira letra quando o Spark não tem o ícone), o título da página e, depois dele em cinza, o site (`youtube.com`). Passar o mouse destaca a linha, e o título mostra o endereço completo numa dica.
- **Uma linha vertical curta** entre duas visitas do mesmo dia com mais de 30 minutos de diferença, para separar as sessões de navegação.
- **Carregar mais** no fim acrescenta as próximas 60 visitas.

## Ações

- **Clique num título** para abrir a página numa nova guia (o painel fecha).
- **⋮ numa linha** abre um menu: **Mais deste site** preenche a pesquisa com esse site, e **Excluir do histórico** apaga essa visita.
- **As caixas de seleção** marcam visitas; **Shift**+clique marca o intervalo desde a última caixa marcada. Com algo selecionado, uma barra substitui o cabeçalho com a quantidade, **✕** (limpar a seleção), **Abrir** (abre cada página selecionada numa guia própria) e **Excluir** (apaga as visitas selecionadas). Clicar num título com algo selecionado marca a visita em vez de abrir.
- **Esc** fecha primeiro o menu da linha, depois limpa a seleção e por fim fecha o painel. As setas andam pelo menu.

Apagar visitas mantém o resto da lista como está (as páginas já carregadas ficam), e a página de nova aba e as sugestões da barra de endereço deixam de mostrá-las.

## Como funciona

- O painel é uma web view nativa sobre a página (`public/history-overlay.html`, `public/overlays/history-overlay.js` e `.css`), desenhada pelo shell (`HistoryDrawer.svelte`) com as linhas prontas: `history-rows.ts` decide o dia de cada visita, o site, a chave do ícone e as pausas de mais de meia hora. Os ícones dos sites vêm do cache de ícones como endereços `data:image/`, uma vez por site.
- As ações voltam ao Rust pela ponte do painel (`window/history_overlay/bridge.rs`), que só aceita `close`, `search`, `clear`, `load-more`, `open`, `open-many`, `delete` e `more-from-site`, e só para visitas que o painel mostra. Uma seleção tem no máximo 1.200 visitas.
- `delete_history_entries` apaga visitas pelo id numa única transação (no máximo 2.000 por chamada) e atualiza a página de nova aba. Uma falha é registrada no log de erros e mostrada no painel.
- O painel aceita até 1.200 visitas, então **Carregar mais** continua funcionando depois da primeira página.

Testes: `tests/history-panel.test.mjs` (dias, pausas, registro, a barra de seleção e o menu), os testes da ponte em `bridge.rs` (abrir, abrir várias, apagar e mais deste site só aceitam visitas mostradas) e `history_repo.rs` (apaga só as visitas escolhidas).
