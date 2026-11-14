# Modo escuro nos sites

**Configurações → Aparência e disposição → Modo escuro nos sites** escurece os sites que estão claros. Vem **desligado por padrão** e vale para todas as abas ao mesmo tempo, sem recarregar.

## O que faz e o que não faz

- Age **somente em páginas claras**. Depois que a página carrega, o navegador lê a cor realmente pintada atrás do conteúdo (o fundo da página ou a tela branca, quando a página não pinta nada). Se for clara, a página é escurecida. Uma página que já é escura, ou que passou sozinha para um tema escuro pela preferência do sistema, fica exatamente como o autor fez.
- A decisão é verificada de novo quando a página troca o próprio tema (mudanças de classe ou estilo na raiz ou no corpo da página). Assim, um site que muda para o tema escuro perde o filtro em vez de ser invertido de volta para claro.
- O escurecimento inverte as cores da página e gira o matiz de volta, então as cores de destaque ficam próximas das originais. Imagens, vídeo, canvas, objetos embutidos e quadros são invertidos de volta para que as fotos mantenham suas cores. Efeitos colaterais: logotipos desenhados em preto podem ficar escuros sobre escuro, e imagens de fundo são invertidas junto com a página.
- Funciona igual no Windows e no Linux, porque é aplicado por um script na página, e não por uma opção do motor. Só páginas de nível superior são tratadas; as telas do próprio navegador não.
- O script não tem como falar com o aplicativo; a interface o liga com um token por visualização, do mesmo jeito que o salto automático de anúncios.

## Armazenamento

A escolha é salva neste dispositivo (como os outros botões de aparência) e não entra nos backups. O lado nativo a guarda só em memória, e a interface a envia de novo ao iniciar.

## Desempenho

Na máquina de teste, rolar um artigo longo e uma página de notícias pesada com o filtro ligado manteve o mesmo ritmo de quadros que sem ele (nenhum quadro acima de 33 ms nos dois casos). Placas de vídeo mais fracas não foram medidas.

## Relacionado

O estudo de viabilidade original está em [Modo escuro forçado em páginas: investigação de viabilidade](forced-dark-mode-investigation.md).
