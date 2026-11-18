# Paletas de cores

[English](../../en/features/color-palettes.md)

As cores de destaque do navegador podem ser trocadas em **Configurações → Aparência e disposição → Paleta de cores**. O destaque é a cor da aba selecionada, dos botões e interruptores ativos, dos realces e dos anéis de foco.

## Paletas

| Paleta | Tema escuro | Tema claro |
| --- | --- | --- |
| **Grafite** (padrão) | cinza claro | quase preto |
| **Oceano** | azul | azul mais profundo |
| **Violeta** | violeta | violeta mais profundo |
| **Esmeralda** | verde | verde mais profundo |
| **Rosa** | rosa | rosa mais profundo |
| **Âmbar** | âmbar | âmbar mais profundo |

Grafite é o preto e cinza originais do navegador e é o que uma instalação nova usa. Ele não acrescenta nada à página: as outras paletas só trocam as cores de destaque, então os fundos, o texto e o estilo da janela (Fosco, Vidro, Gelo) continuam como estão. Cada paleta tem uma versão mais clara para o tema escuro e uma mais profunda para o tema claro, para o texto por cima continuar legível.

## Paleta e cores dos workspaces

Cada workspace tem as próprias cores de destaque (Configurações → Workspaces e tema). A paleta é o destaque do **navegador inteiro**, então, enquanto você usa uma paleta diferente do Grafite, ela substitui as cores dos workspaces em todo lugar. Com o **Grafite** (o padrão) as cores dos workspaces valem exatamente como antes, então é a opção a usar se você quer uma cor para cada workspace. Trocar a paleta repinta o navegador na hora.

## Como funciona

- A escolha é salva neste dispositivo e não faz parte dos backups.
- **Vale para todas as janelas ao mesmo tempo**: as janelas que já estão abertas mudam assim que você escolhe uma paleta, sem precisar reabri-las.
- **Chega ao navegador inteiro**: os painéis que abrem como camadas separadas (ferramentas, downloads, configurações) copiam as cores do navegador, e a página de nova guia também segue a paleta (anéis de foco, opções selecionadas, interruptores e controles deslizantes).
- **O botão do escudo** mostra o estado "ligado" no destaque da paleta. O Grafite mantém o verde original, então a aparência padrão não muda.
- **Cores com significado continuam fixas**: o roxo das janelas privadas e das abas efêmeras, o verde e o vermelho dos estados de download e backup, e os erros.
- A paleta é um único atributo na página (`data-palette`); as cores ficam em [`src/styles/theme.css`](../../../src/styles/theme.css). Para acrescentar uma paleta, inclua-a em `PALETTES` em [`src/lib/services/palette-preference.js`](../../../src/lib/services/palette-preference.js), dê a ela um bloco escuro e um claro no `theme.css`, uma amostra no `settings.css` e um nome nos catálogos de idioma.
