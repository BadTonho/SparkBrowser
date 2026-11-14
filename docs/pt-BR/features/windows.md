# Várias janelas do navegador

Abra outra janela com **Ctrl+N** ou pelo menu **⋯ Mais** → **Nova janela**. Cada janela tem a sua barra de abas, barra de endereço, aba selecionada, workspace selecionado e divisão de tela, então dá para deixar uma em cada monitor. Cookies, logins, favoritos, histórico, downloads, configurações e workspaces são compartilhados por todas as janelas.

Isto é diferente de uma [janela privada](./private-window.md): ela é outro processo e não guarda nada. As janelas normais compartilham um único perfil.

## Mover abas entre janelas

Clique com o botão direito em uma aba e escolha **Mover para a janela N** ou **Mover para uma nova janela**. A aba passa a ser a selecionada no destino; a janela de onde ela saiu seleciona uma vizinha (ou uma página de nova aba, se era a última). A página recarrega na nova janela.

## Fechar

- Fechar a **primeira janela** fecha o navegador, com a revisão de [encerramento protegido](./shutdown.md) de sempre. A revisão cobre as páginas de todas as janelas.
- Fechar **outra janela** nunca apaga as abas dela: elas vão para as [abas fechadas recentemente](./closed-tabs.md), de onde podem ser reabertas.
- Fechar a última aba de uma janela deixa uma página de nova aba, como na primeira janela.

## O que volta na próxima abertura

O Spark grava cada janela com as suas abas, a aba e o workspace que estavam selecionados, e onde ela estava na tela (tamanho, posição, maximizada e monitor). Na próxima abertura **todas as janelas reabrem**, também depois de uma queda ou de o processo ser morto à força. Isso protege as abas que ficam em uma janela que não é a primeira.

- As janelas voltam à mesma posição no Windows e no Linux com X11. No **Wayland** quem escolhe onde a janela aparece é o compositor, então o Spark restaura as abas, o tamanho e o estado maximizado, mas não o monitor.
- Uma posição que nenhum monitor mostra mais (uma tela que foi desligada) é ignorada, para a janela nunca reabrir fora do alcance.
- Se uma janela não conseguir reabrir, as abas dela vão para as abas fechadas recentemente e o navegador mostra o erro. Nada some em silêncio.

## Painéis e menus

Os popovers (ferramentas, downloads, configurações, histórico, privacidade, permissões, zoom e mídia), o seletor de workspace e o menu de favoritos abrem dentro da janela que os pediu. O vídeo em tela cheia ocupa só a própria janela.

## Testes

```
node tools/testing/native-smoke.mjs --case browser-windows
node tools/testing/native-smoke.mjs --case window-restore
```

`browser-windows` abre uma segunda janela, confere que as abas continuam separadas, move uma aba, prepara os painéis nas duas janelas e fecha a janela. `window-restore` reinicia o app, primeiro matando o processo e depois com uma saída normal, e confere que as duas janelas voltam com as suas abas.
