# Trabalho de otimização

As mudanças exigem validação local antes da release. O comportamento nativo também deve ser verificado nos sistemas operacionais de destino; verificações automatizadas sozinhas não comprovam latência da interface ou economia de memória.

## Geometria e IPC

As medições do viewport são agrupadas por frame de animação. Há apenas uma solicitação de resize em andamento, com retenção da medida mais recente e descarte de atualizações idênticas já concluídas. Atualizações de geometria não aguardam operações de abas nem bloqueios do banco. A reconciliação completa do ciclo de vida usa um mapa de rótulos das WebViews em vez de buscas lineares repetidas.

## Sessões e ordenação

O salvamento de sessão usa upserts incrementais transacionais com instruções preparadas: linhas inalteradas não são regravadas e somente abas removidas são excluídas. O arraste altera uma prévia local, confirma uma vez ao soltar e restaura a ordem ao cancelar ou falhar. Solicitações de reordenação são serializadas. Histórico e downloads possuem índices cronológicos; a hibernação adaptativa obtém os IDs de abas frequentes em uma consulta agrupada e remove eventos expirados na passagem periódica. Falhas de leitura de linhas e de gravação de ativações são retornadas.

Callbacks nativos de navegação enfileiram gravações de histórico/sessão em um worker. Consultas/limpeza do histórico e encerramento normal aguardam as gravações anteriores. Comandos explícitos de abas persistem antes de retornar. O worker não imprime URLs de páginas nos erros.

Criação, seleção, ordenação, fixação, movimentação de abas e alterações do limite residente restauram um checkpoint se a persistência falhar. O registro de uso da seleção e as linhas da sessão são confirmados na mesma transação. Os documentos nativos são reconciliados após a persistência; uma falha posterior de exibição mostra um erro e preserva a sessão confirmada. O rollback preserva observações de navegação/mídia recebidas durante a operação.

Comandos de banco para favoritos, notas, histórico e busca de comandos usam um executor bloqueante com duas permissões de worker e o mutex da conexão existente. A espera pelo lock e a execução SQL possuem medições separadas. As permissões limitam tarefas bloqueantes ativas; não adicionam conexões nem garantem ordem FIFO entre solicitações concorrentes. Gravações de navegação mantêm seu worker limitado e suas regras de flush.

## Navegação e diálogos

Rascunhos do endereço são preservados durante a edição e redefinidos na troca explícita de aba ou ao pressionar Escape. Eventos nativos de carregamento e título substituem os temporizadores fixos do indicador. Observadores do WebView2 no Windows fornecem mudanças de endereço (incluindo URLs de SPA), disponibilidade do histórico e falhas de navegação; outros motores atualmente deixam a disponibilidade do histórico desconhecida. O indicador HTTPS descreve o esquema, não uma auditoria do certificado.

Snapshots de abas possuem revisões monotônicas; respostas antigas são ignoradas. Atualizações de navegação por aba possuem revisão própria e preservam metadados mais recentes ao receber snapshots completos. Tokens de geração rejeitam callbacks de views substituídas. Diálogos compartilham foco inicial, contenção do foco, circulação com Tab, Escape e restauração, incluindo diálogos sobrepostos e gravação de atalhos.

## Busca e armazenamento de preferências

O histórico pesquisa todas as visitas armazenadas com páginas limitadas, escape de curingas literais e cursor estável `(visited_at, id)`. A interface agrupa a digitação e descarta respostas atrasadas. O LIKE usa as regras de correspondência de maiúsculas do SQLite; não é um índice Unicode de texto completo. Favoritos são agrupados por pasta uma vez por alteração.

Sugestões de comandos são calculadas somente com a barra aberta, classificadas pela relevância de título/URL, normalizadas para acentos e limitadas por categoria. O acesso às preferências usa um adaptador que trata armazenamento negado e falta de espaço, mantém o estado em memória e avisa o usuário sem registrar valores armazenados.

A busca persistente de comandos normaliza o texto na gravação e mantém índices transacionais de trigramas SQLite FTS5 para notas, nomes de sessões e páginas de sessões. Consultas com pelo menos três caracteres Unicode usam frases literais no índice; consultas menores percorrem os campos já normalizados. A busca obtém três correspondências por categoria, retorna duas e indica se existem outras, sem calcular totais exatos. Os índices exigem armazenamento e trabalho adicional na gravação. A busca do painel de notas mantém a semântica LIKE existente e retorna até 100 resumos com prévias de 180 caracteres; abrir uma nota busca seu texto completo.

O gravador de notas remove rascunhos salvos antigos acima de 32 entradas ou 2.000.000 de unidades UTF-16, preservando editores ativos e rascunhos não salvos, em gravação ou com falha. A importação por arquivo mantém o documento selecionado no Rust com um token opaco, libera-o na substituição/cancelamento/importação e calcula a impressão da prévia diretamente em SHA-256. Relatórios de mídia sem alterações deixam de incrementar revisões ou publicar snapshots completos; a consulta periódica continua para descobrir vídeos.

## Memória residente e migrações

Um limite flexível configurável de abas residentes (padrão 8, intervalo 1–100) hiberna as abas elegíveis menos usadas recentemente. Abas ativas, fixadas, carregando, com formulário alterado, mídia reproduzindo ou download são protegidas; o alvo pode ser ultrapassado quando necessário. A fixação é mantida na restauração da sessão. A hibernação automática ignora abas protegidas e evita persistência/reconciliação quando nada mudou. O limite residente continua válido com o temporizador de inatividade desligado.

A detecção de atividade é uma tentativa para formulários DOM e áudio/vídeo do documento principal; não garante detectar estado de iframes, Web Audio, editores personalizados ou trabalho não salvo específico da aplicação. Fixe páginas importantes explicitamente. Somente sinalizadores booleanos são transmitidos, com verificação da geração de cada view.

Adições de esquema e índices cronológicos usam migrações transacionais nomeadas. O `user_version` de paletas existente é importado para o registro compartilhado sem reaplicar uma migração concluída. Testes cobrem atualização de sessões antigas, idempotência, rollback e restauração de fixação.

## Downloads

Downloads manuais por URL são tarefas com revisão, progresso, cancelamento e fila limitada a 64 tarefas pendentes. Dois slots de semáforo compartilhados cobrem transferências manuais e iniciadas por páginas. Clientes HTTP são reutilizados por política do escudo, incluindo validação de redirecionamentos. O cancelamento cooperativo verifica cada leitura; uma operação de rede bloqueada pode levar até seu timeout de 15 segundos para terminar. Arquivos manuais parciais são removidos no cancelamento/falha e arquivos existentes nunca são substituídos.

Downloads nativos das páginas mantêm autenticação do motor e suporte a blobs, com nomes de destino reservados e exclusivos. Quando ambos os slots estão ocupados, downloads de páginas são recusados com aviso visível para tentar novamente, em vez de serem repetidos sem cookies. O hook portátil do Tauri não expõe progresso/cancelamento nativo; essas tarefas exibem apenas estado. Transferências concluídas são registradas separadamente no SQLite. O encerramento normal cancela tarefas manuais e permite até 20 segundos para limpeza. A finalização abrupta do processo não garante limpeza.

## Segurança local e política do escudo em execução

Comandos da aplicação são declarados no manifesto de build do Tauri e concedidos explicitamente somente à WebView local `main`. Permissões para a janela inteira e padrões amplos do core foram removidos. Uma verificação adicional de rótulo no handler rejeita WebViews filhas mesmo que naveguem para recursos locais. A interface local e os overlays usam CSP com scripts hospedados localmente; atributos de estilo inline continuam permitidos para layout dinâmico. Essa política não substitui a CSP dos sites remotos.

O Windows substitui seu script de política registrado no WebView2 para documentos futuros e atualiza os interceptadores da página atual, preservando formulários, posição de rolagem e histórico nativo. Alterações malsucedidas tentam restaurar a política anterior. Outros motores usam o fallback de recriação documentado, recusam alterações enquanto formulários/mídia/downloads detectados estiverem protegidos e exibem o aviso de reinicialização. A filtragem continua sendo uma tentativa; transferências de rede existentes não são desfeitas retroativamente.

## Fronteiras dos módulos e inicialização

A fachada da bridge delega para módulos de transporte de navegação, overlays, workspaces, downloads, persistência, privacidade, eventos e controles da janela. Armazenamento/migrações do preview de workspaces são separados do IPC nativo e validam os perfis restaurados. A exclusão no preview recebe as abas atuais em vez de inventar uma sessão inicial.

Categorias de configurações, gravação de atalhos, catálogo de busca e CSS compartilhado restrito ao modal possuem módulos separados. Hooks de páginas, roteamento de teclado e reconciliação do ciclo de vida têm módulos Rust próprios. Listeners são registrados antes da inicialização; registros tardios são removidos após a desmontagem. O writer de navegação tem fila limitada a 256 mensagens e agrupa solicitações que salvam somente a sessão.

## Validação local e nativa

As verificações são executadas localmente e no fluxo de CI de pull requests e da branch principal; o fluxo de release separado compila artefatos assinados. Os cenários nativos WebDriver continuam separados. Veja [testes nativos](../contributing/native-tests.md) para configuração e escopo. O comportamento nativo deve ser validado em cada plataforma. DPI entre monitores, latência interativa e comparações de RAM agregada continuam como trabalho de medição; compilação ou contagens de views não comprovam ganhos.

## Trabalho em repouso e armazenamento

O pop-up de permissões acompanha a posição somente quando necessário e publica mudanças de geometria. Páginas no volume padrão não agendam trabalho de volume por alterações irrelevantes do DOM; com volume diferente do padrão, a observação de players dinâmicos continua, assim como a consulta periódica da aba ativa. A coleta de contextos dos diagnósticos não mantém o bloqueio de abas enquanto espera pela interface, e atualizações simultâneas do painel compartilham uma solicitação.

As leituras de sites recentes compartilham um cache por revisão da nova guia, mudanças na conexão SQLite e `data_version`; mudanças no histórico, ícones e gravações externas provocam recálculo. O mesmo resultado limitado revalida os destinos dos cartões. A sincronização de extensões no Windows usa a conexão inicializada, e arquivos e diretórios são processados em um executor bloqueante limitado. Painéis ocultos carregam independentemente no primeiro uso. Os registros do papel de parede e a conexão IndexedDB são reutilizados; novas imagens são salvas como um único blob e URLs de dados antigas continuam legíveis. Resultados sintéticos e a opção de compilação nativa rápida estão em [validação de desempenho](../contributing/performance.md).

## Diagnósticos locais

As configurações exibem amostras limitadas de tempo de IPC em memória, contagens acumuladas de chamadas/erros, duração de operações nativas, linhas de sessão gravadas e contagens de abas residentes/protegidas. Os contadores de tempo omitem argumentos, URLs, títulos e tokens. O inventário local separado `get_diagnostics.tab_details` contém URLs/títulos das abas e IDs de workspace; a resposta também contém o diretório local de dados da aplicação. Os contadores de recursos informam quantidades de tokens de bridge, scripts do escudo, entradas de mídia e contextos de execução do Windows sem expor seus identificadores. Remova inventários/caminhos locais antes de compartilhar uma resposta de diagnóstico. Veja [validação de desempenho](../contributing/performance.md) para comparações reproduzíveis e limites de medição.

## Inicialização, polling e armazenamento

As listas de filtro em cache são reaproveitadas enquanto têm menos de 24 horas. A atualização em segundo plano espera o tempo restante, em vez de baixar e recompilar a cada abertura do app; um cache ausente é atualizado imediatamente. A compilação do motor continua síncrona na inicialização, então a proteção por filtros vale desde a primeira requisição. Movê-la para fora da thread principal exige antes um formato serializado do motor e medições.

O painel de popover lê as variáveis de tema só depois que o estilo, as classes ou os atributos de tema do elemento raiz mudam, em vez de chamar `getComputedStyle` a cada frame. Enquanto um popover está aberto, o acompanhamento da âncora ainda usa um callback por frame, porque transformações CSS podem mover o botão sem redimensioná-lo.

O polling de mídia continua para descobrir vídeos. Ele roda a cada 3 segundos quando a aba em foco não tem áudio, vídeo nem frames tocando, e a cada segundo quando há mídia. Por isso, um vídeo adicionado a uma página ociosa pode levar até 3 segundos para aparecer.

As conexões SQLite definem um tempo limite de ocupado, armazenamento temporário em memória, um cache de páginas de 8 MB e um limite de memória mapeada de 256 MB. As consultas de listagem de histórico e favoritos usam instruções preparadas em cache.
