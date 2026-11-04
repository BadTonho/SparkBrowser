# Painel flutuante de downloads e progresso na barra

Downloads abre 8 px abaixo do botão, alinhado à direita e limitado por margens de 8 px da janela. A largura é de 480 px, limitada ao espaço disponível; a altura acompanha o conteúdo até 560 px ou o espaço restante da janela. Listas longas rolam internamente. O cartão tem fundo opaco do tema (`--bg-downloads-panel`); somente a superfície nativa ao redor permanece transparente. As páginas nativas e ambos os painéis da divisão ficam visíveis ao redor do cartão, sem escurecer, recarregar ou perder formulários e rolagem.

Clique novamente no botão, clique fora, pressione Escape ou use Fechar para dispensar. O primeiro clique externo apenas fecha o painel. Fechar pelo teclado/cabeçalho devolve o foco ao botão. Outro painel, perda de foco do aplicativo ou remoção da referência fecha Downloads sem recuperar o foco. Atalhos resolvem Downloads ou Mais no modo compacto e restauram a barra no modo Zen.

O formulário de URL, SHA-256 opcional expansível, histórico, Pausar, Retomar, Reiniciar, Cancelar e verificar continuam disponíveis. O estado vazio exige ausência tanto de tarefas quanto de histórico.

## Borda de progresso

Transferências com tamanho conhecido usam `soma dos bytes recebidos limitados / soma dos bytes totais` das tarefas atualmente em Downloading. Pausados e encerrados ficam fora do cálculo. Alterar o conjunto ativo recalcula o percentual; não é uma porcentagem permanente de um lote. Qualquer total desconhecido/não positivo torna a borda indeterminada.

Sem transferências, tarefas Queued/Verifying/Finalizing indicam processamento sem percentual. Quando restarem somente Paused/Pausing, uma borda amarela e estática mantém o progresso combinado conhecido ou congela o segmento para totais desconhecidos. Downloads concluídos geram uma borda verde completa com uma breve animação; resultados Failed/VerificationFailed geram uma borda vermelha completa e estática. Cancelamentos não criam avisos de erro. A prioridade é erro não visto, transferência, processamento, pausa, conclusão não vista e, por último, ausência de indicador. Reconhecer um erro devolve a borda ao estado das tarefas restantes.

Os avisos verde e vermelho persistem após reiniciar o aplicativo até abrir Downloads pelo botão, atalho ou item do menu. Clicar no botão com o painel aberto também reconhece os avisos e fecha o painel. Novos resultados com o painel aberto geram novos avisos; Fechar pelo cabeçalho, Escape e cliques externos comuns não reconhecem esses resultados. Mais recebe a mesma borda no modo compacto, com estado traduzido na entrada Downloads; apenas abrir Mais não reconhece os avisos.

Um snapshot inicial e os eventos existentes atualizam o indicador com o painel fechado. O histórico anterior à implantação estabelece uma referência inicial sem avisos. Um registro separado em `safeStorage`, na chave `veloce_download_notices`, armazena somente IDs, revisões, estados observados e avisos pendentes, nunca URLs, nomes ou caminhos dos arquivos. Ele impede atualizações antigas/repetidas e persiste o reconhecimento; dados inválidos reiniciam a referência inicial, e falhas de armazenamento preservam o estado da sessão com o aviso de armazenamento existente. Tarefas removidas são retiradas do registro. Esse registro da interface não entra nos backups nem altera o armazenamento das transferências.

O SVG usa a cor de destaque do tema durante transferências e as variáveis CSS `--download-status-completed`, `--download-status-paused` e `--download-status-error` para os estados de resultado. Ele respeita movimento reduzido e não intercepta o ponteiro nem altera a reorganização dos atalhos. Tooltips e descrições acessíveis traduzidas identificam o estado sem depender apenas da cor.

## Arquitetura e validação

Downloads e Ferramentas compartilham renderização/transporte seguro de apresentação, com serviços nativos, documentos, rotas, tokens e contadores de sequência separados. Controladores e operações permanecem na interface principal. O documento auxiliar recebe uma árvore de apresentação e envia ações tipadas para identificadores opacos de controles; não recebe permissões para comandos do aplicativo. Campos desconhecidos/duplicados, dados/tokens inválidos, sequências repetidas e sessões encerradas são rejeitados. Elementos nativos de progresso são aceitos sem marcação executável ou estilos inline.

A prévia web compartilha estilos e geometria. Armazenamento, concorrência, regras de retomada e backups permanecem iguais. Execute `npm run test:native -- --case downloads-popover` para o painel e `--case downloads` para transferências/reinício/integridade. Veja o [comportamento dos downloads](downloads.md) e os [testes nativos](../contributing/native-tests.md).
