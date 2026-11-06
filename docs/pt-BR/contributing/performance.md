# Validação de desempenho

Abra **Settings → Local Diagnostics** para inspecionar as contagens de IPC, falhas, duração média e p95 recente da sessão, além das contagens nativas de gravação de sessão e das WebViews realmente residentes. A atualização é manual. Os contadores ficam em memória, retêm no máximo 256 amostras de tempo e não contêm URLs, títulos, argumentos de comandos ou tokens. O p95 cobre as amostras ainda retidas, não toda a sessão.

O inventário separado `get_diagnostics.tab_details` inclui URLs/títulos de abas locais e IDs de workspace. A resposta também inclui o diretório de dados da aplicação; remova esses campos antes de compartilhá-la. `native_resources` contém somente contagens de recursos: tokens de bridge, scripts do escudo, entradas de mídia e views/contextos de execução de mídia do Windows. As contagens de contexto do Windows são `null` quando indisponíveis. Compare contagens estabilizadas com as views residentes ao investigar criação/fechamento de abas; contagens limitadas não comprovam heaps limitados do WebView2.

## Comparações reproduzíveis

Use o mesmo build de release, computador, escala da tela e páginas locais de teste nas duas revisões. Registre commit, sistema operacional, versões de Rust/Node, versão do motor WebView e modo de energia. Aqueça a página de teste uma vez e repita cada execução pelo menos cinco vezes.

| Cenário | Medição | Comportamento esperado |
| --- | --- | --- |
| Recolher/expandir a barra lateral repetidamente | Chamadas de `sync_viewport_bounds` e duração recente | No máximo um resize em andamento; limites idênticos já aplicados são descartados |
| Arrastar uma aba sobre várias vizinhas | Chamadas de `session.persist` e linhas gravadas | Um comando de reordenação ao soltar, nenhum por movimento do ponteiro |
| Selecionar a mesma aba novamente | Linhas de sessão gravadas | Nenhuma linha inalterada é regravada |
| Alternar entre abas carregadas | Duração de IPC e tempo nativo de `tabs.switch` | Sem nova navegação de uma view residente existente |
| Abrir 10, 50 e 100 abas | Views residentes e memória da árvore de processos | Limite flexível respeitado, salvo quando abas protegidas exigem mais views |
| Buscar em mais de 60 visitas do histórico | Continuidade dos resultados e latência | Páginas do cursor sem duplicatas ou perda de visitas com o mesmo timestamp |
| Buscar uma coleção grande de notas/sessões | Duração da consulta, tamanho dos índices e custo de gravação | Mesmos resultados literais/Unicode; prévias limitadas; `2+` para correspondências adicionais |

`tabs.switch` mede o trabalho do backend incluindo a espera pelo bloqueio de operações; IPC inclui também transporte e agendamento do frontend. Nenhuma das métricas mede a primeira pintura. Para medir o primeiro conteúdo, use uma página local com um marcador visível e meça da ação do usuário até esse marcador por ferramentas de integração nativa.

Execute a comparação sintética explícita com `cargo test --manifest-path src-tauri/Cargo.toml --offline --lib compare_indexed_search_with_review_baseline -- --ignored --nocapture`. Ela cria em memória 5.000 notas e 1.000 sessões de três páginas, aquece cada consulta uma vez e imprime cinco medições pareadas contra o SQL da revisão analisada. Ela verifica os mesmos IDs retornados, sem um limite de tempo dependente do ambiente. Um benchmark debug não representa a latência da interface nativa release; registre o perfil de compilação com os resultados. Veja as [otimizações implementadas](../architecture/optimization-work.md) para o desenho atual.

## Memória e binários

No Windows, some o working set da aplicação Rust e de todos os processos WebView2 descendentes. Em outras plataformas, inclua também os processos auxiliares do motor. Aguarde o mesmo tempo de estabilização em cada cenário. Contagem de WebViews não mede RAM.

Compile com `npm run build` e `cargo build --release --manifest-path src-tauri/Cargo.toml`. O perfil de release atual usa thin LTO, uma unidade de codegen e remoção de símbolos; compare tamanho do binário, tempo de build limpo, inicialização e p95 de troca de abas antes de alterá-lo. `npm run test:native:build:fast` gera um build de teste isolado opcional com menor otimização, sem LTO e com mais unidades de codegen; ele não substitui o build nativo de release usado na validação de distribuição. Um binário cargo de release não é um instalador assinado.

Execute `cargo test --manifest-path src-tauri/Cargo.toml --offline --lib measure_session_snapshot_scaling -- --ignored --nocapture` para medir salvamentos de sessão sem alterações para 10, 100 e 1.000 abas artificiais em SQLite em memória. O teste verifica que nenhuma linha muda e imprime a mediana de dez execuções. Uma execução local debug mediu aproximadamente 0,062 ms, 0,426 ms e 4,37 ms, respectivamente; esses números não representam disco ou latência da interface. A implementação transacional incremental foi mantida até que uma comparação próxima da produção justifique uma alternativa mais complexa.

O helper da interface nativa não inicializou no ambiente de implementação (`failed to write kernel assets`, erro 3 do sistema operacional). A suíte WebDriver separada no Windows passou nas verificações de foco do DOM e ciclo de vida; veja [integração nativa](native-tests.md). Isso não comprova latência interativa, RAM agregada, DPI entre monitores ou primeira pintura. Resultados de testes nativos devem ser relatados separadamente de compilação e testes unitários.
