# Responsabilidade pelo estado e contratos de operação

`BrowserState` é uma fachada sobre armazenamento privado de sessão, políticas, superfícies, segurança e infraestrutura. Os nomes IPC e os dados serializados existentes permanecem iguais. Não há migração de banco ou backup.

## Sessão e observações dos documentos

Os comandos de organização ficam em `state/session_ops`; `commands/` mantém fachadas de compatibilidade. Somente o módulo de estado pode obter acesso mutável ao armazenamento de sessão. As demais funcionalidades recebem consultas com acesso somente de leitura. O guard de operação de sessão serializa mudanças de organização, incluindo as transferências de estado do backup e da inicialização.

Use `TabCheckpoint` para mudanças reversíveis: capture os metadados, aplique a mudança, ajuste visibilidade e residência, persista e então altere os documentos nativos e emita o snapshot confirmado. Sua extensão transacional pode incluir gravações de outro domínio; uma falha reverte tanto o banco quanto a organização em memória. Os marcadores de exclusão de contas e as mudanças de sessão usam essa fronteira. Nunca mantenha um guard síncrono de estado durante um await ou callback nativo.

Navegação nativa, mídia e downloads informam mudanças por `observe_document`. Sua entrada expõe somente URL, título, navegação e atividade. Ela não permite mover abas, alterar contas, selecionar workspaces ou recriar abas removidas. O serviço incrementa a revisão da observação somente quando há mudança. A transferência de estado de uma importação preserva observações recebidas durante a operação no banco.

As políticas expõem consultas por valor ou guards somente de leitura e setters explícitos. Os serviços de superfícies e segurança mantêm portas de sincronização com escopo para os handlers de janelas nativas; essas portas não concedem escrita na organização da sessão. As conexões de repositórios continuam pertencendo à infraestrutura e devem seguir as regras existentes de executor e transação do banco.

## Ciclo do conteúdo nativo

O serviço de ciclo de conteúdo controla criação, reconciliação e limpeza; sua fábrica privada constrói as WebViews de cada plataforma. Os imports existentes de `webview_manager` continuam válidos pela fachada. A reconciliação reutiliza documentos existentes. Fechamento e reinicialização usam o mesmo caminho de limpeza para permissões pendentes, downloads nativos, sessões de mídia, bridges de páginas internas e tokens de teclado.

Cada documento residente registra seu perfil resolvido. A reconciliação compara esse vínculo com os metadados confirmados, incluindo abas ocultas após remover um workspace. Ela recria documentos somente quando necessário; mover uma aba de container entre workspaces preserva o documento porque o perfil do container permanece igual.

No Windows, a primeira navegação aguarda a instalação dos handlers nativos na WebView em branco. Pedidos recebidos durante a instalação substituem o destino pendente, impedindo que uma inicialização atrasada sobrescreva a URL solicitada pelo usuário. A conclusão é vinculada à geração da WebView; callbacks de documentos fechados ou recriados não podem abrir um destino antigo.

Persista mudanças de perfil antes de fechar o documento afetado. Mudanças apenas de geometria usam operações de viewport e não devem reconstruir páginas. Falhas nativas após o commit devem ser relatadas como falhas de exibição; não trate uma transação já confirmada como revertida.

No Windows, notificações passivas de atividade e mensagens da nova aba interna usam o registro existente de contextos de execução nativos para não cancelar uma navegação em andamento. O receptor confere contexto vivo, origem atual, geração, limite e formato do envelope, canal permitido e parâmetros autenticados existentes. As mensagens aguardam a disponibilização do binding no documento. O Linux mantém o transporte por navegação. Nenhum dos caminhos concede permissões IPC do aplicativo aos sites.

## Responsabilidade no frontend

A fachada do navegador delega para ações de sessão, estado confirmado privado, edição de endereço e apresentação. As stores públicas de abas, URL confirmada, seleção de workspace, lista de workspaces e layout dividido expõem assinaturas, sem setters. Os consumidores usam ações; a verificação de arquitetura restringe imports de mutação privados.

A ordem de arraste é uma camada de apresentação sobre as abas confirmadas. Cancelar ou falhar um arraste remove essa camada, impedindo o retorno de uma aba fechada recentemente. Uma falha ao mover de workspace preserva a sessão confirmada mais recente. As revisões de snapshots rejeitam respostas antigas de comandos; observações recentes dos documentos sobrevivem a snapshots de organização anteriores.

O ciclo do navegador registra listeners antes das leituras iniciais e ignora respostas após o descarte. Os ciclos compartilhados de domínio mantêm um único conjunto de assinaturas e timers por contexto JavaScript, com contagem de consumidores e limpeza idempotente. Assinaturas registradas tardiamente são liberadas imediatamente. A inicialização de workspaces não sobrescreve uma seleção recebida por evento mais recente. Mídia, downloads, permissões, backup, zoom, PiP, atualizações, encerramento, arraste, atalhos de workspace e preferências da página interna usam esse mecanismo.

Os painéis de mídia, zoom e privacidade coordenam a exclusão mútua por um pequeno coordenador de superfícies. Suas stores não importam umas às outras. Uma abertura atrasada perde a prioridade quando outro painel é solicitado.

Painéis projetados descartam ações de foco de apresentações antigas e ignoram o fechamento de uma sessão anterior após reabrir. Downloads mantém um pedido de foco pendente quando o envio recolhe o formulário antes de habilitar seu botão; atualizações comuns de progresso não roubam o foco.

Consulte o [fluxo de regressões](../contributing/regression-workflow.md) para as verificações automáticas e os cenários obrigatórios.
