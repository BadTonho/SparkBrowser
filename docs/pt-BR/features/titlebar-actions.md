# Reorganizar atalhos rápidos

Segure o botão esquerdo do mouse sobre um atalho à direita da barra superior por meio segundo e arraste na horizontal. O botão destacado acompanha o ponteiro e uma linha vertical indica a posição de inserção. Solte dentro da faixa para salvar a ordem automaticamente para as próximas aberturas.

Um clique rápido continua abrindo a função. Pressione **Esc**, saia da janela ou solte fora da faixa para cancelar. Mover mais de 6 pixels antes de completar meio segundo cancela o gesto e impede um clique acidental.

A ordem é compartilhada entre os layouts de abas horizontal e vertical. O atalho de favoritos mantém sua posição salva enquanto estiver oculto no layout vertical. O menu compacto **Mais** mantém a ordem existente. Os controles da janela permanecem fixos.

A preferência é salva localmente neste dispositivo e não faz parte das exportações de backup. Se o armazenamento estiver indisponível, a nova ordem continua disponível durante a sessão atual.

## Escolher quais botões aparecem

Por padrão, só **Configurações**, **Downloads**, **Proteção** e **Extensões** ficam na barra superior. Todos os outros botões (Ferramentas, Barra de favoritos, Comandos, Histórico, Modo Zen, QR Code e, no Windows, WhatsApp) ficam desligados até você ligá-los. Abra **Configurações** (a engrenagem) e use **Botões da barra** nas configurações rápidas: cada botão é um interruptor com o seu ícone. Um botão desligado mantém a sua posição, então volta ao mesmo lugar quando é ligado de novo. O botão Configurações não pode ser ocultado, porque é ele que abre este menu. Da esquerda para a direita, a ordem padrão é Extensões, Proteção, Downloads e Configurações; os botões que você ligar aparecem à esquerda deles, e você ainda pode arrastá-los para qualquer lugar. A escolha é salva localmente neste dispositivo e não entra nos backups.

## Aparência dos ícones

Os dez atalhos usam SVGs decorativos de contorno, com tamanho de 16 × 16 pixels e traço uniforme de 1,8 unidade. Ferramentas usa uma lista, favoritos uma barra, comandos um raio, histórico um relógio, downloads uma seta para baixo, privacidade um escudo, configurações uma engrenagem e foco quatro cantos de enquadramento.

Os ícones herdam a propriedade CSS `color` do botão por meio de `stroke="currentColor"`. Altere o `color` do botão ou as variáveis existentes do tema (`--text-secondary`, `--text-primary` ao passar o mouse e `--accent-primary` para favoritos ativos) para recolori-los. O indicador verde de privacidade permanece separado. As cores dos ícones não possuem um seletor nas configurações.


## QR Code

A ação QR Code abre um painel local para a página HTTP(S) ativa. Ela gera um PNG sem serviço de rede e permite copiar a imagem ou salvá-la pelo diálogo nativo. Se a cópia falhar, o painel informa o erro e mantém a opção de salvar. Páginas internas e endereços acima da capacidade do formato exibem uma explicação. A ação pode ser reordenada e sua posição é salva junto à faixa da barra de título.