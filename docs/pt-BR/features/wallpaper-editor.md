# Editor de plano de fundo personalizado

[English](../../en/features/wallpaper-editor.md)

**Personalizar a nova guia → Plano de fundo → Personalizado → Personalizar** abre o editor em uma guia própria (`spark://wallpaper`). É uma página própria do navegador: ela não consegue chamar nenhum comando do navegador, e tudo o que desenha e lê fica na página.

## Layout

- **Barra da esquerda:** duas abas. **Opções** lista os looks: **Nebula**, **Space** e **Galaxy**. **Presets** é onde você guarda suas configurações: **Salvar preset** guarda, com um nome, uma cópia de tudo o que você ajustou, clicar em um preset mostra ele de novo e ✕ exclui.
- **Barra da direita:** as configurações do look escolhido.
- **Centro:** o plano de fundo ao vivo, desenhado com WebGL e atualizado a cada ajuste.

## Os looks

Cada look tem suas próprias configurações de **Forma** e **Movimento**, e a barra da direita muda quando você escolhe outro look. Os valores de cada look são guardados separadamente: ao voltar a um look, os controles estão como você deixou. Cores, Mapa de gradiente, Correção e Efeitos (abaixo) valem para todos.

- **Nebula:** nuvens que se movem, com um campo de estrelas que cintilam. Forma: Escala, Ondulação, Contraste, Estrelas. Movimento: Velocidade, Direção (para frente ou para trás) e Ângulo (o rumo em que as nuvens se movem).
- **Space:** estrelas em profundidade, vindo na sua direção a velocidades diferentes sobre um brilho suave. Forma: Densidade, Tamanho, Cintilação, Profundidade (quanto as camadas diferem em velocidade), Brilho de fundo. Movimento: Velocidade, Direção (na sua direção ou para longe) e Balanço (o campo todo se move de um lado para o outro).
- **Galaxy:** uma espiral girando em torno de um núcleo brilhante, com faixas de poeira e estrelas. Forma: Braços, Torção, Núcleo, Poeira, Estrelas, Inclinação (quanto o disco é visto de lado). Movimento: Velocidade de rotação, Direção (horário ou anti-horário) e Respiração (a espiral aperta e afrouxa devagar).

## Configurações comuns a todos os looks

- **Cores:** oito amostras (clique em uma para escolher a cor), **Aleatório**, **De uma imagem** (lê uma imagem do seu disco e guarda as oito cores principais; a imagem não é enviada para lugar nenhum) e **Cores ativas**, quantas das oito o gradiente usa.
- **Mapa de gradiente:** **Mistura** (sRGB, Linear, OKLab ou OKLCH), **Paradas** (arraste uma alça, ou foque nela e use as setas, com Shift para passos maiores), **Viés** e **Nitidez**. **Redefinir** restaura esta seção.
- **Correção:** Brilho, Contraste e Saturação.
- **Efeitos:** **Granulação** (Clássica ou Filme, quantidade, tamanho, cor, foco nos tons médios, semente e animação), **Tom** (Nenhum ou ACES, exposição em EV), **Vinheta** e **Arte com dither** (dither ordenado). **Redefinir** restaura esta seção.

## Usar um preset como plano de fundo da nova guia

Um preset lembra com qual look foi feito. Cada preset que você salva aparece como um cartão em **Personalizar a nova guia → Plano de fundo → Personalizado**, com uma prévia pequena das cores. Clique em um cartão para usá-lo como plano de fundo da página de nova guia (ele é desenhado ao vivo, atrás do relógio e dos widgets); clique de novo no cartão escolhido para voltar ao plano de fundo de imagem ou vídeo. Enquanto há um plano de fundo personalizado escolhido, ele substitui o de imagem ou vídeo. As páginas de nova guia abertas se atualizam sozinhas quando você escolhe um preset ou quando salva um novo no editor. Uma cópia das configurações fica guardada junto com a sua escolha, então excluir o preset no editor não muda o plano de fundo.

## Limites por enquanto

Os presets e o plano de fundo escolhido ficam neste dispositivo, no armazenamento das próprias páginas, e não entram nos backups. O que você está editando não é salvo sozinho; a barra de cima (salvar, exportar, predefinições) e os outros looks vêm depois. O desenho pausa enquanto a guia está escondida e é limitado a uma densidade de pixels de 1,5 para ficar leve. Se a janela do navegador não tiver WebGL, a página avisa em vez de desenhar.
