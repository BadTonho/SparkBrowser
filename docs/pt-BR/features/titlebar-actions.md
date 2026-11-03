# Reorganizar atalhos rápidos

Segure o botão esquerdo do mouse sobre um atalho à direita da barra superior por meio segundo e arraste na horizontal. O botão destacado acompanha o ponteiro e uma linha vertical indica a posição de inserção. Solte dentro da faixa para salvar a ordem automaticamente para as próximas aberturas.

Um clique rápido continua abrindo a função. Pressione **Esc**, saia da janela ou solte fora da faixa para cancelar. Mover mais de 6 pixels antes de completar meio segundo cancela o gesto e impede um clique acidental.

A ordem é compartilhada entre os layouts de abas horizontal e vertical. O atalho de favoritos mantém sua posição salva enquanto estiver oculto no layout vertical. O menu compacto **Mais** mantém a ordem existente. Os controles da janela permanecem fixos.

A preferência é salva localmente neste dispositivo e não faz parte das exportações de backup. Se o armazenamento estiver indisponível, a nova ordem continua disponível durante a sessão atual.

## Aparência dos ícones

Os oito atalhos usam SVGs decorativos de contorno, com tamanho de 16 × 16 pixels e traço uniforme de 1,8 unidade. Ferramentas usa uma lista, favoritos uma barra, comandos um raio, histórico um relógio, downloads uma seta para baixo, privacidade um escudo, configurações uma engrenagem e foco quatro cantos de enquadramento.

Os ícones herdam a propriedade CSS `color` do botão por meio de `stroke="currentColor"`. Altere o `color` do botão ou as variáveis existentes do tema (`--text-secondary`, `--text-primary` ao passar o mouse e `--accent-primary` para favoritos ativos) para recolori-los. O indicador verde de privacidade permanece separado. As cores dos ícones não possuem um seletor nas configurações.
