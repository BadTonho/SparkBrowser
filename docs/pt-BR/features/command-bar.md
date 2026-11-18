# Aparência da Barra de Comandos

[English](../../en/features/command-bar.md)

A Barra de Comandos (**Ctrl + K** por padrão) abre como uma camada acima da página. **A página continua visível atrás dela**, escurecida, em vez de ser trocada por uma tela escura, então você mantém o seu lugar enquanto busca ou executa um comando. O que ela busca e executa está descrito em [Notas e sessões na Barra de Comandos](./command-search.md).

## Como ela é

- Usa a mesma superfície, borda e sombra dos outros painéis do navegador, então acompanha o tema (escuro ou claro), o estilo do fundo da janela (Fosco, Vidro, Gelo) e a paleta de cores.
- É limpa: uma linha de entrada, uma lista com um leve tom na linha selecionada (sem contorno, brilho nem deslocamento), selos pequenos e neutros e um rodapé de uma linha com as teclas.
- O campo de texto comporta o texto de ajuda ("Buscar, ir a um endereço ou executar um comando").

## Como funciona

- Dentro do aplicativo a barra é desenhada por uma camada nativa acima da página. A barra de verdade fica na janela do navegador, fora da tela, e guarda toda a lógica; a camada mostra uma cópia dela e repassa o que você digita, as setas, o **Enter** e os cliques.
- **Esc**, **Ctrl + K**, um clique fora da barra ou a troca para outra janela a fecham. Ao fechar, o teclado volta para a janela do navegador.
- A camada é preparada alguns segundos depois de o navegador iniciar e mantida por três minutos após cada uso, então abri-la pelo atalho é rápido.
- Fora do aplicativo (a interface vista num navegador comum) não há camada nativa, então a barra é desenhada na página como antes.
