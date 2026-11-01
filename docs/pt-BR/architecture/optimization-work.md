# Trabalho de otimização

As mudanças são entregues em commits separados e validados. O comportamento nativo também deve ser verificado nos sistemas operacionais de destino; verificações automatizadas sozinhas não comprovam latência da interface ou economia de memória.

## Geometria e IPC

As medições do viewport são agrupadas por frame de animação. Há apenas uma solicitação de resize em andamento, com retenção da medida mais recente e descarte de atualizações idênticas já concluídas. Atualizações de geometria não aguardam operações de abas nem bloqueios do banco. A reconciliação completa do ciclo de vida usa um mapa de rótulos das WebViews em vez de buscas lineares repetidas.

## Etapas restantes

- Persistência incremental de sessão, índices SQL e confirmação de arraste.
- Rascunhos de endereço, metadados de navegação e acessibilidade de modais.
- Busca no histórico, classificação de comandos e armazenamento confiável.
- Limite de WebViews residentes, tarefas de download e fronteiras de segurança locais.
- Modularização, diagnósticos, validação contínua e orientações de release com medições.
