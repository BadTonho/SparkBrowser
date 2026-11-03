# Correções de falhas — 2 de novembro de 2026

Estas mudanças tratam as cinco falhas reproduzidas após a análise de otimizações. Este fluxo não cria commit.

| Falha | Correção | Cobertura de regressão |
| --- | --- | --- |
| A política do shield mudava antes de uma gravação de sessão falhar | No Windows, a persistência da sessão termina antes de mudar a política em execução. O motor alternativo planeja e persiste os metadados antes de reconstruir documentos. | A rejeição de gravação SQLite no teste nativo preserva política, contagem de carregamentos e histórico de navegação; novas tentativas cobrem os dois sentidos da alternância. Rust verifica o rollback dos metadados no motor alternativo. |
| A exclusão do workspace era confirmada antes de falhar o registro de ativação | Remoção do workspace, abas da sessão, divisões e registro de ativação compartilham uma transação SQLite. O estado só é publicado após o commit. | A rejeição de ativação no teste nativo preserva workspace, sessão, registros de uso e aba ativa; novas tentativas cobrem abas de destino existentes e recém-criadas, além da migração bem-sucedida de abas. |
| Uma substituição de divisão malsucedida destruía o histórico de uma página residente | Planejar e persistir metadados da divisão/abas antes de reconciliar WebViews nativas. | A rejeição de inserção no teste nativo com limite de duas views preserva o par original, contagens de carregamento e histórico de retorno, seguida de uma nova tentativa bem-sucedida. |
| A falha ao salvar a proporção deixava memória e geometria adiantadas em relação ao SQLite | Separar a prévia de arraste das divisões confirmadas; restaurar a proporção confirmada e a geometria nativa após falha na gravação. | Rust verifica isolamento da prévia e rollback. Os testes nativos medem largura dos painéis, persistência de outra ação durante a prévia, commits malsucedidos/bem-sucedidos e cancelamento. |
| Uma busca contendo NUL causava erro no parser FTS | Encaminhar consultas com NUL para o fallback literal `instr` existente sobre o texto normalizado. | Rust cobre notas, páginas de sessões, consultas curtas e resultados vazios; a busca nativa cobre notas salvas. |

Erros de reconciliação nativa após o commit usam o evento de erro visível existente e preservam o snapshot confirmado, para que a interface represente a mudança salva. Falhas de transação ocorrem antes de operações nativas destrutivas.

A exclusão de workspace também preserva a regra da divisão no destino: fechar um workspace ativo retorna ao painel focado lembrado pelo destino; transferir sua aba ativa para uma divisão substitui somente o painel focado. As regressões nativas incluem outra aba sem par no destino para detectar a seleção acidental de uma terceira página visível.

A documentação agora inclui o guia de desempenho em inglês que faltava e remove links para arquivos de revisão ausentes neste checkout. As descrições das funcionalidades e o inventário de cenários nativos estão sincronizados em inglês e português.

## Validação

`npm run validate` passou. Após o ajuste final de foco na divisão ao excluir workspace, a formatação Rust, o Clippy com `-D warnings` e todos os testes Rust passaram novamente. As verificações JavaScript/documentação e a compilação nativa isolada também passaram.

| Verificação | Resultado |
| --- | --- |
| JavaScript | 65 aprovados |
| Svelte/TypeScript | 0 erros, 0 avisos |
| Testes unitários Rust | 81 aprovados, 1 benchmark explícito ignorado |
| Integração Rust do atualizador | 1 aprovado, 1 teste de instalador assinado ignorado (exige compilação separada dos testes de instalador) |
| Regressões nativas de persistência | Cenários de divisão, workspace e shield aprovados na build Windows isolada final; workspace inclui o destino dividido |
| Busca nativa e restauração de funcionalidades | Busca de nota com NUL aprovada; restauração de divisão/sessão aprovada, incluindo nova execução final das funcionalidades |
| FPS do PiP | Todas as seis amostras de dez segundos aprovadas: aproximadamente 30 FPS e 59,5–59,7 FPS nos vídeos artificiais de 30/60 FPS; intervalos máximos de até 53,7 ms, sem perdas informadas na origem |

A execução nativa completa aprovou inicialmente 24 de 28 cenários. Um callback tardio de navegação atingiu o trigger artificial de falha na gravação de sessão do teste de shield; o teste agora reconhece esse erro esperado da fila antes de tentar novamente, e a nova execução do shield passou. O cenário principal de PiP também passou na nova execução após uma interrupção inicial por falta de foco em primeiro plano. Os resultados dependentes de foco no desktop são registrados separadamente abaixo.

Todos os 28 cenários tiveram execuções aprovadas entre a suíte completa e as repetições direcionadas. O cenário de sombra passou separadamente. O teclado passou com ativação explícita da janela isolada, agora incluída em sua preparação; uma repetição posterior separada foi interrompida quando outro navegador assumiu o primeiro plano. As verificações de destino da entrada continuam ativas em cada tecla. Isso não significa uma única execução ininterrupta com os 28 cenários aprovados: o foco em primeiro plano permaneceu instável nesta sessão de desktop.

Logs locais ficam no diretório ignorado `.cache/`: `correctness-validation.log`, `correctness-rust-final.log`, `correctness-native.log`, `correctness-native-recheck.log`, `correctness-native-focus.log`, `correctness-native-keyboard.log` e `correctness-native-shadow.log`. Os artefatos dos cenários preservam relatórios de aprovação/falha; os diretórios data/profile/config/cache/logs são limpos depois.

A execução nativa do motor alternativo em macOS/Linux exige essas plataformas; o rollback de seus metadados é coberto aqui por testes Rust. Estas verificações não comprovam a ausência de qualquer outro defeito possível no navegador.
