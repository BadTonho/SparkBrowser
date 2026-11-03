# Painel flutuante de Ferramentas

Ferramentas abre 8 pixels abaixo do botão na barra, com a borda direita alinhada ao botão e pelo menos 8 pixels de margem nas bordas da janela. O painel tem no máximo 640 pixels de largura e altura; conteúdos longos possuem rolagem interna. Na tela dividida compacta, ele abre abaixo de **Mais**. Aberturas pela barra de comandos usam a mesma referência, e abrir Ferramentas no modo Zen restaura a barra primeiro.

As cinco abas reúnem diagnóstico de abas, sessões nomeadas, organização de abas, notas por página e abas fechadas recentemente. Use as setas Esquerda/Direita ou Home/End nas abas e Tab para percorrer os controles. A aba selecionada tem destaque discreto, com texto legível, campos compactos e ícone de fechar.

Clique fora, clique novamente no botão, pressione Esc ou use o ícone de fechar para encerrar. Cliques externos apenas fecham o painel. Esc e o ícone de fechar devolvem o foco ao botão de origem. A posição acompanha o redimensionamento da janela e a reorganização da barra. Remover o botão de referência, abrir outro painel do navegador ou sair do aplicativo fecha Ferramentas.

## Apresentação nativa e controle dos dados

No desktop, uma WebView transparente dedicada apresenta o painel acima das duas páginas nativas. Ferramentas não oculta, recarrega nem substitui as páginas. Os demais painéis do navegador mantêm seu comportamento.

A interface principal mantém os controladores Svelte das ferramentas, as operações de dados, as confirmações e a fila única de salvamento de notas. Ela projeta uma árvore de apresentação estruturada com elementos permitidos e identificadores opacos de controles para uma superfície Svelte pequena, usando a mesma marcação e os mesmos estilos da prévia web. A árvore transporta texto e atributos seguros, nunca HTML executável. A superfície envia somente ações tipadas de controles por uma ponte autenticada por sessão; ela não pode executar comandos do aplicativo. Sequências antigas, formatos de ação desconhecidos e tokens inválidos são rejeitados. Apresentações sem mudanças não são reenviadas, e edições aguardando confirmação não são substituídas por estados antigos.

As notas mantêm o intervalo de salvamento, a recuperação de erros e os rascunhos protegidos quando o painel fecha. O painel não cria um segundo gravador de notas nem outro formato de armazenamento.

## Validação

Execute `npm run test:native -- --case tools-popover` após compilar o aplicativo nativo isolado. O cenário verifica o posicionamento, o modo compacto, as cinco seções, a edição nativa de sessões e notas, a preservação do cursor, ações forjadas, a negação de IPC direto e a preservação das duas páginas divididas. Consulte a [integração nativa](../contributing/native-tests.md).
