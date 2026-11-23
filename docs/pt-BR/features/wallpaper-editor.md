# Editor de plano de fundo personalizado

[English](../../en/features/wallpaper-editor.md)

**Personalizar a nova guia → Plano de fundo → Personalizado → Personalizar** abre o editor em uma guia própria (`spark://wallpaper`). É uma página própria do navegador: ela não consegue chamar nenhum comando do navegador, e tudo o que desenha e lê fica na página.

## Layout

- **Barra da esquerda:** duas abas. **Opções** lista os looks (**Nebula** é o primeiro). **Presets** é onde você guarda suas configurações: **Salvar preset** guarda, com um nome, uma cópia de tudo o que você ajustou, clicar em um preset mostra ele de novo e ✕ exclui.
- **Barra da direita:** as configurações do look escolhido.
- **Centro:** o plano de fundo ao vivo, desenhado com WebGL e atualizado a cada ajuste.

## Nebula

Nuvens que se movem e um campo de estrelas que cintilam, coloridas por um gradiente que você monta.

- **Cores:** oito amostras (clique em uma para escolher a cor), **Aleatório**, **De uma imagem** (lê uma imagem do seu disco e guarda as oito cores principais; a imagem não é enviada para lugar nenhum) e **Cores ativas**, quantas das oito o gradiente usa.
- **Mapa de gradiente:** **Mistura** (sRGB, Linear, OKLab ou OKLCH), **Paradas** (arraste uma alça, ou foque nela e use as setas, com Shift para passos maiores), **Viés** e **Nitidez**. **Redefinir** restaura esta seção.
- **Forma:** Escala, Ondulação, Contraste e Estrelas. **Movimento:** Deriva, a velocidade das nuvens. **Correção:** Brilho, Contraste e Saturação.
- **Efeitos:** **Granulação** (Clássica ou Filme, quantidade, tamanho, cor, foco nos tons médios, semente e animação), **Tom** (Nenhum ou ACES, exposição em EV), **Vinheta** e **Arte com dither** (dither ordenado). **Redefinir** restaura esta seção.

## Usar um preset como plano de fundo da nova guia

Cada preset que você salva aparece como um cartão em **Personalizar a nova guia → Plano de fundo → Personalizado**, com uma prévia pequena das cores. Clique em um cartão para usá-lo como plano de fundo da página de nova guia (ele é desenhado ao vivo, atrás do relógio e dos widgets); clique de novo no cartão escolhido para voltar ao plano de fundo de imagem ou vídeo. Enquanto há um plano de fundo personalizado escolhido, ele substitui o de imagem ou vídeo. As páginas de nova guia abertas se atualizam sozinhas quando você escolhe um preset ou quando salva um novo no editor. Uma cópia das configurações fica guardada junto com a sua escolha, então excluir o preset no editor não muda o plano de fundo.

## Limites por enquanto

Os presets e o plano de fundo escolhido ficam neste dispositivo, no armazenamento das próprias páginas, e não entram nos backups. O que você está editando não é salvo sozinho; a barra de cima (salvar, exportar, predefinições) e os outros looks vêm depois. O desenho pausa enquanto a guia está escondida e é limitado a uma densidade de pixels de 1,5 para ficar leve. Se a janela do navegador não tiver WebGL, a página avisa em vez de desenhar.
