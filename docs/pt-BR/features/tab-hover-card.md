# Cartão de prévia da guia

Deixar o mouse sobre uma guia por cerca de meio segundo abre um cartão ao lado dela com:

- o nome da guia, resumido em uma linha (no máximo 60 caracteres);
- uma imagem da página;
- a memória que a guia usa, ou a memória que ela economiza quando está em repouso.

O cartão abre abaixo da guia com as guias horizontais e à direita dela com as guias verticais, e sempre fica dentro da janela. Com um cartão aberto, passar para a guia seguinte mostra o cartão dela na hora. Clicar em uma guia, rolar, começar a arrastar, abrir o menu da guia, sair da guia ou trocar para outro aplicativo fecha o cartão. Ele nunca pega o foco nem o mouse, então as guias embaixo dele continuam clicáveis. Ele substitui a dica de texto que as guias mostravam antes.

## Imagens das páginas

Só dá para fotografar uma página que está na tela, então o Spark tira a imagem:

- da página que você está vendo, cerca de 1,5 segundo depois de trocar para ela;
- das páginas na tela enquanto o mouse está sobre as guias (no máximo a cada 3 segundos), para que a guia que você vai deixar tenha uma imagem recente;
- da guia sob o mouse, quando ela está na tela (o cartão mostra primeiro a última imagem e depois atualiza).

Uma guia em segundo plano ou em repouso mostra como a página estava da última vez que apareceu na tela. Uma guia que ainda não apareceu nesta execução mostra "Sem prévia ainda".

As imagens são pequenas (528 pixels de largura; JPEG no Windows, PNG no Linux) e ficam só na memória: nunca são gravadas no disco, na sessão ou em backups. Outras guias nunca são carregadas nem acordadas para tirar uma imagem. O Spark guarda no máximo 40, descarta primeiro as usadas há mais tempo e esquece a imagem de uma guia fechada.

## Como funciona

As páginas são web views separadas que cobrem a interface do navegador, então um cartão desenhado pela interface ficaria escondido atrás da página. Como a prévia de arrastar guias, o cartão é desenhado por uma janela transparente própria (`tab-hover-card`). Essa janela tem só o tamanho do cartão, fica ao lado da guia, ignora o mouse e aparece sem ser ativada. Passar de uma guia para a seguinte mantém a janela na tela e só troca o conteúdo e a posição. Quando o cartão fecha, o conteúdo é apagado na hora, a janela é escondida meio segundo depois (para reabrir de um quadro vazio, nunca com o cartão anterior) e é destruída depois de um minuto sem passar o mouse nas guias, para não manter uma web view na memória.

- Interface: a action `tabHoverCard` (`src/lib/actions/tabHoverCard.ts`) cuida do atraso e do fechamento, e envia os pedidos por `src/lib/services/bridge/tab-hover-card.ts`. Cada pedido tem uma revisão crescente, então um pedido atrasado nunca reabre um cartão já fechado.
- Rust: `src-tauri/src/window/tab_hover_card/` guarda o estado do cartão e as imagens (`mod.rs`), tira as imagens (`capture.rs`) e cuida da janela (`surface.rs`). Comandos: `set_tab_hover_card` e `refresh_tab_previews`.
- Imagens: no Windows, o WebView2 executa o método do DevTools `Page.captureScreenshot` sem recorte nem escala, e o Spark reduz a imagem. Com recorte ou escala, o Chromium desenha a página de novo em outro tamanho para a captura, o que faz a página na tela piscar. No Linux, o WebKitGTK tira uma captura da região visível, reduzida com o cairo.
- A página do cartão (`public/tab-hover-card.html`) escreve os textos como texto e só mostra imagens `data:image/`.
- O cartão não aparece com um painel do navegador aberto, durante o arraste de uma guia ou enquanto o Spark está fechando.
- Uma imagem que falha é registrada no log de erros como Warning, com a guia a que se referia, e o cartão mantém a imagem anterior.
