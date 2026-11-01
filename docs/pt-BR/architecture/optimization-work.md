# Trabalho de otimização

As mudanças são entregues em commits separados e validados. O comportamento nativo também deve ser verificado nos sistemas operacionais de destino; verificações automatizadas sozinhas não comprovam latência da interface ou economia de memória.

## Geometria e IPC

As medições do viewport são agrupadas por frame de animação. Há apenas uma solicitação de resize em andamento, com retenção da medida mais recente e descarte de atualizações idênticas já concluídas. Atualizações de geometria não aguardam operações de abas nem bloqueios do banco. A reconciliação completa do ciclo de vida usa um mapa de rótulos das WebViews em vez de buscas lineares repetidas.

## Sessões e ordenação

O salvamento de sessão usa upserts incrementais transacionais com instruções preparadas: linhas inalteradas não são regravadas e somente abas removidas são excluídas. O arraste altera uma prévia local, confirma uma vez ao soltar e restaura a ordem ao cancelar ou falhar. Solicitações de reordenação são serializadas. Histórico e downloads possuem índices cronológicos; a hibernação adaptativa obtém os IDs de abas frequentes em uma consulta agrupada e remove eventos expirados na passagem periódica. Falhas de leitura de linhas e de gravação de ativações são retornadas.

Callbacks nativos de navegação enfileiram gravações de histórico/sessão em um worker. Consultas/limpeza do histórico e encerramento normal aguardam as gravações anteriores. Comandos explícitos de abas persistem antes de retornar. O worker não imprime URLs de páginas nos erros.

## Navegação e diálogos

Rascunhos do endereço são preservados durante a edição e redefinidos na troca explícita de aba ou ao pressionar Escape. Eventos nativos de carregamento e título substituem os temporizadores fixos do indicador. Observadores do WebView2 no Windows fornecem mudanças de endereço (incluindo URLs de SPA), disponibilidade do histórico e falhas de navegação; outros motores atualmente deixam a disponibilidade do histórico desconhecida. O indicador HTTPS descreve o esquema, não uma auditoria do certificado.

Snapshots de abas possuem revisões monotônicas; respostas antigas são ignoradas. Atualizações de navegação por aba possuem revisão própria e preservam metadados mais recentes ao receber snapshots completos. Tokens de geração rejeitam callbacks de views substituídas. Diálogos compartilham foco inicial, contenção do foco, circulação com Tab, Escape e restauração, incluindo diálogos sobrepostos e gravação de atalhos.

## Busca e armazenamento de preferências

O histórico pesquisa todas as visitas armazenadas com páginas limitadas, escape de curingas literais e cursor estável `(visited_at, id)`. A interface agrupa a digitação e descarta respostas atrasadas. O LIKE usa as regras de correspondência de maiúsculas do SQLite; não é um índice Unicode de texto completo. Favoritos são agrupados por pasta uma vez por alteração.

Sugestões de comandos são calculadas somente com a barra aberta, classificadas pela relevância de título/URL, normalizadas para acentos e limitadas por categoria. O acesso às preferências usa um adaptador que trata armazenamento negado e falta de espaço, mantém o estado em memória e avisa o usuário sem registrar valores armazenados.

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

## Validação contínua e nativa

CI verifica Windows, Linux e macOS, com um job WebDriver separado e isolado no Windows. A suíte local de release no Windows passou na preservação de abas/formulário/rolagem, histórico SPA, atualização do shield, proteção de memória, rascunhos de endereço, foco de diálogos, resize e restauração de sessão. Veja [testes nativos](../contributing/native-tests.md) para configuração e escopo. Jobs remotos de CI e runtimes de outras plataformas não foram executados no ambiente de implementação. DPI entre monitores, latência interativa e comparações de RAM agregada continuam como trabalho de medição; compilação ou contagens de views não comprovam ganhos.

## Diagnósticos locais

As configurações exibem amostras limitadas de tempo de IPC em memória, contagens acumuladas de chamadas/erros, duração de operações nativas, linhas de sessão gravadas e contagens de abas residentes/protegidas. Os dados de diagnóstico omitem argumentos, URLs, títulos e tokens. Veja [validação de desempenho](../contributing/performance.md) para comparações reproduzíveis e limites de medição.
