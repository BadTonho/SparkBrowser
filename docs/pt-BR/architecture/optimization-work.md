# Trabalho de otimização

As mudanças são entregues em commits separados e validados. O comportamento nativo também deve ser verificado nos sistemas operacionais de destino; verificações automatizadas sozinhas não comprovam latência da interface ou economia de memória.

## Geometria e IPC

As medições do viewport são agrupadas por frame de animação. Há apenas uma solicitação de resize em andamento, com retenção da medida mais recente e descarte de atualizações idênticas já concluídas. Atualizações de geometria não aguardam operações de abas nem bloqueios do banco. A reconciliação completa do ciclo de vida usa um mapa de rótulos das WebViews em vez de buscas lineares repetidas.

## Sessões e ordenação

O salvamento de sessão usa upserts incrementais transacionais com instruções preparadas: linhas inalteradas não são regravadas e somente abas removidas são excluídas. O arraste altera uma prévia local, confirma uma vez ao soltar e restaura a ordem ao cancelar ou falhar. Solicitações de reordenação são serializadas. Histórico e downloads possuem índices cronológicos; a hibernação adaptativa obtém os IDs de abas frequentes em uma consulta agrupada e remove eventos expirados na passagem periódica. Falhas de leitura de linhas e de gravação de ativações são retornadas.

Callbacks nativos de navegação enfileiram gravações de histórico/sessão em um worker. Consultas/limpeza do histórico e encerramento normal aguardam as gravações anteriores. Comandos explícitos de abas persistem antes de retornar. O worker não imprime URLs de páginas nos erros.


## Navega??o e di?logos

Rascunhos do endere?o s?o preservados durante a edi??o e redefinidos na troca expl?cita de aba ou ao pressionar Escape. Eventos nativos de carregamento e t?tulo substituem os temporizadores fixos do indicador. Observadores do WebView2 no Windows fornecem mudan?as de endere?o (incluindo URLs de SPA), disponibilidade do hist?rico e falhas de navega??o; outros motores atualmente deixam a disponibilidade do hist?rico desconhecida. O indicador HTTPS descreve o esquema, n?o uma auditoria do certificado.

Snapshots de abas possuem revis?es monot?nicas; respostas antigas s?o ignoradas. Atualiza??es de navega??o por aba possuem revis?o pr?pria e preservam metadados mais recentes ao receber snapshots completos. Tokens de gera??o rejeitam callbacks de views substitu?das. Di?logos compartilham foco inicial, conten??o do foco, circula??o com Tab, Escape e restaura??o, incluindo di?logos sobrepostos e grava??o de atalhos.

## Busca e armazenamento de prefer?ncias

O hist?rico pesquisa todas as visitas armazenadas com p?ginas limitadas, escape de curingas literais e cursor est?vel `(visited_at, id)`. A interface agrupa a digita??o e descarta respostas atrasadas. O LIKE usa as regras de correspond?ncia de mai?sculas do SQLite; n?o ? um ?ndice Unicode de texto completo. Favoritos s?o agrupados por pasta uma vez por altera??o.

Sugest?es de comandos s?o calculadas somente com a barra aberta, classificadas pela relev?ncia de t?tulo/URL, normalizadas para acentos e limitadas por categoria. O acesso ?s prefer?ncias usa um adaptador que trata armazenamento negado e falta de espa?o, mant?m o estado em mem?ria e avisa o usu?rio sem registrar valores armazenados.

## Mem?ria residente e migra??es

Um limite flex?vel configur?vel de abas residentes (padr?o 8, intervalo 1?100) hiberna as abas eleg?veis menos usadas recentemente. Abas ativas, fixadas, carregando, com formul?rio alterado, m?dia reproduzindo ou download s?o protegidas; o alvo pode ser ultrapassado quando necess?rio. A fixa??o ? mantida na restaura??o da sess?o. A hiberna??o autom?tica ignora abas protegidas e evita persist?ncia/reconcilia??o quando nada mudou. O limite residente continua v?lido com o temporizador de inatividade desligado.

A detec??o de atividade ? uma tentativa para formul?rios DOM e ?udio/v?deo do documento principal; n?o garante detectar estado de iframes, Web Audio, editores personalizados ou trabalho n?o salvo espec?fico da aplica??o. Fixe p?ginas importantes explicitamente. Somente sinalizadores booleanos s?o transmitidos, com verifica??o da gera??o de cada view.

Adi??es de esquema e ?ndices cronol?gicos usam migra??es transacionais nomeadas. O `user_version` de paletas existente ? importado para o registro compartilhado sem reaplicar uma migra??o conclu?da. Testes cobrem atualiza??o de sess?es antigas, idempot?ncia, rollback e restaura??o de fixa??o.

## Downloads

Downloads manuais por URL s?o tarefas com revis?o, progresso, cancelamento e fila limitada a 64 tarefas pendentes. Dois slots de sem?foro compartilhados cobrem transfer?ncias manuais e iniciadas por p?ginas. Clientes HTTP s?o reutilizados por pol?tica do escudo, incluindo valida??o de redirecionamentos. O cancelamento cooperativo verifica cada leitura; uma opera??o de rede bloqueada pode levar at? seu timeout de 15 segundos para terminar. Arquivos manuais parciais s?o removidos no cancelamento/falha e arquivos existentes nunca s?o substitu?dos.

Downloads nativos das p?ginas mant?m autentica??o do motor e suporte a blobs, com nomes de destino reservados e exclusivos. Quando ambos os slots est?o ocupados, downloads de p?ginas s?o recusados com aviso vis?vel para tentar novamente, em vez de serem repetidos sem cookies. O hook port?til do Tauri n?o exp?e progresso/cancelamento nativo; essas tarefas exibem apenas estado. Transfer?ncias conclu?das s?o registradas separadamente no SQLite. O encerramento normal cancela tarefas manuais e permite at? 20 segundos para limpeza. A finaliza??o abrupta do processo n?o garante limpeza.

## Etapas restantes

- Busca no histórico, classificação de comandos e armazenamento confiável.
- Limite de WebViews residentes, tarefas de download e fronteiras de segurança locais.
- Modularização, diagnósticos, validação contínua e orientações de release com medições.
