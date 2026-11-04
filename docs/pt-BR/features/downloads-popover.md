# Painel flutuante de downloads e progresso na barra

Downloads abre 8 px abaixo do botão, alinhado à direita e limitado por margens de 8 px da janela. A largura é de 480 px, limitada ao espaço disponível; a altura acompanha o conteúdo até 560 px ou o espaço restante da janela. Listas longas rolam internamente. As páginas nativas e ambos os painéis da divisão ficam visíveis, sem escurecer, recarregar ou perder formulários e rolagem.

Clique novamente no botão, clique fora, pressione Escape ou use Fechar para dispensar. O primeiro clique externo apenas fecha o painel. Fechar pelo teclado/cabeçalho devolve o foco ao botão. Outro painel, perda de foco do aplicativo ou remoção da referência fecha Downloads sem recuperar o foco. Atalhos resolvem Downloads ou Mais no modo compacto e restauram a barra no modo Zen.

O formulário de URL, SHA-256 opcional expansível, histórico, Pausar, Retomar, Reiniciar, Cancelar e verificar continuam disponíveis. O estado vazio exige ausência tanto de tarefas quanto de histórico.

## Borda de progresso

Transferências com tamanho conhecido usam `soma dos bytes recebidos limitados / soma dos bytes totais` das tarefas atualmente em Downloading. Pausados e encerrados ficam fora do cálculo. Alterar o conjunto ativo recalcula o percentual; não é uma porcentagem permanente de um lote. Qualquer total desconhecido/não positivo torna a borda indeterminada.

Sem transferências, tarefas Queued/Verifying/Finalizing indicam processamento sem percentual. Quando restarem somente Paused/Pausing, uma borda discreta e estática mostra o progresso combinado conhecido ou um segmento fixo para totais desconhecidos. Outros estados encerrados/interrompidos permanecem no gerenciador sem manter a borda ativa.

Mais recebe a mesma borda no modo compacto, com estado traduzido na entrada Downloads. Um snapshot inicial e os eventos existentes atualizam o indicador com o painel fechado. O SVG usa a cor do tema, respeita movimento reduzido e não intercepta o ponteiro nem altera a reorganização dos atalhos.

## Arquitetura e validação

Downloads e Ferramentas compartilham renderização/transporte seguro de apresentação, com serviços nativos, documentos, rotas, tokens e contadores de sequência separados. Controladores e operações permanecem na interface principal. O documento auxiliar recebe uma árvore de apresentação e envia ações tipadas para identificadores opacos de controles; não recebe permissões para comandos do aplicativo. Campos desconhecidos/duplicados, dados/tokens inválidos, sequências repetidas e sessões encerradas são rejeitados. Elementos nativos de progresso são aceitos sem marcação executável ou estilos inline.

A prévia web compartilha estilos e geometria. Armazenamento, concorrência, regras de retomada e backups permanecem iguais. Execute `npm run test:native -- --case downloads-popover` para o painel e `--case downloads` para transferências/reinício/integridade. Veja o [comportamento dos downloads](downloads.md) e os [testes nativos](../contributing/native-tests.md).
