# Validação de desempenho

Abra **Settings → Local Diagnostics** para inspecionar as contagens de IPC, falhas, duração média e p95 recente da sessão, além das contagens nativas de gravação de sessão e das WebViews realmente residentes. A atualização é manual. Os contadores ficam em memória, retêm no máximo 256 amostras de tempo e não contêm URLs, títulos, argumentos de comandos ou tokens. O p95 cobre as amostras ainda retidas, não toda a sessão.

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

`tabs.switch` mede o trabalho do backend incluindo a espera pelo bloqueio de operações; IPC inclui também transporte e agendamento do frontend. Nenhuma das métricas mede a primeira pintura. Para medir o primeiro conteúdo, use uma página local com um marcador visível e meça da ação do usuário até esse marcador por ferramentas de integração nativa.

## Memória e binários

No Windows, some o working set da aplicação Rust e de todos os processos WebView2 descendentes. Em outras plataformas, inclua também os processos auxiliares do motor. Aguarde o mesmo tempo de estabilização em cada cenário. Contagem de WebViews não mede RAM.

Compile com `npm run build` e `cargo build --release --manifest-path src-tauri/Cargo.toml`. Compare tamanho do binário, tempo de build limpo, inicialização e p95 de troca de abas antes de ativar thin LTO, alterar unidades de codegen ou remover símbolos. Nenhum perfil personalizado de release é ativado somente por uma melhoria presumida. Um binário cargo de release não é um instalador assinado.

O helper da interface nativa não inicializou no ambiente de implementação (`failed to write kernel assets`, erro 3 do sistema operacional). Portanto, esse ambiente não comprova latência interativa, RAM agregada, foco/DPI ou primeira pintura. Resultados de testes nativos e execuções de CI devem ser relatados separadamente de compilação e testes unitários.
