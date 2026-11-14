# Fronteiras de segurança do aplicativo

## Documentos privilegiados

Comandos do aplicativo exigem tanto o identificador de WebView `main` quanto a origem e o documento esperados da interface. No desenvolvimento, somente a origem configurada do Vite é confiável; no Windows empacotado, utiliza-se a origem dos arquivos internos. Superfícies auxiliares e abas de navegação não recebem permissões para executar comandos do aplicativo. A transição inicial em branco da interface não é privilegiada.

Guardas de navegação no Windows impedem a interface principal de navegar para outros documentos. Painéis internos permanecem nos documentos empacotados e não podem abrir novas janelas. As abas mantêm a navegação HTTP/HTTPS existente e os fluxos explícitos de Nova guia/PiP. Objetos nativos do WebView2 são explicitamente desabilitados e as DevTools só ficam ativas em builds de desenvolvimento; o IPC necessário ao framework permanece disponível sob as permissões do aplicativo. Quando o endurecimento de uma WebView de conteúdo falha na instalação, o navegador encerra em vez de continuar com uma fronteira parcialmente aplicada. Essas fronteiras seguem as [orientações de segurança do WebView2 da Microsoft](https://learn.microsoft.com/en-us/microsoft-edge/webview2/concepts/security).

Eventos internos com estado do navegador são enviados explicitamente à interface principal. Documentos de sites não recebem dados globais de navegação, abas ou permissões.

## Credenciais do ciclo de vida das abas

Cada WebView de conteúdo recebe um token de ponte novo, de 256 bits, na criação. Fechar, hibernar ou reconstruir uma WebView remove o token dela, as sequências aceitas e as sessões de páginas internas, de modo que transações aposentadas não podem ser repetidas por um documento sucessor. A navegação confirmada revoga os privilégios de páginas internas antes de o novo documento carregar, e cada geração de WebView só aceita números de sequência crescentes a partir de um nonce único de documento.

## Mensagens internas

Somente ações e campos reconhecidos são aceitos. A leitura dos parâmetros rejeita nomes decodificados duplicados, campos desconhecidos, credenciais, fragmentos e requisições grandes demais antes de alterar o estado. Dados tipados, tokens da geração/sessão e sequências monotônicas continuam sendo verificados pela ponte responsável. Mensagens de atividade exigem valores booleanos literais; uma entrada inválida não pode remover a proteção de formulário/mídia. Os tokens das pontes contêm 256 bits obtidos da fonte criptográfica do sistema operacional.

Rejeições usam erros fixos e não reproduzem tokens ou corpos de mensagens. O limite da apresentação de Notas comporta o limite existente de 200 KB de texto.

## Manutenção do mecanismo

As distribuições Windows utilizam explicitamente o instalador Evergreen do WebView2. A Microsoft gerencia a instalação e as atualizações do mecanismo; o Spark não muda políticas de atualização do Windows nem baixa o código-fonte do Chromium.

A interface observa `NewBrowserVersionAvailable`, disponibiliza as versões atual e disponível do mecanismo e mostra um aviso de reinício que pode ser dispensado. “Depois” dispensa somente aquela versão disponível na sessão atual; Configurações → Atualizações mantém o reinício pendente. Outra versão disponível pode exibir um novo aviso. Erros de detecção, canais de prévia e mecanismos fixos configurados explicitamente são apresentados como indisponíveis, nunca como prova de que o mecanismo está atualizado. Atualizações podem incluir correções de segurança; o aviso não afirma identificar correções específicas. A Microsoft explica por que adotar uma versão instalada exige [um novo ambiente ou reinício do aplicativo](https://learn.microsoft.com/en-us/microsoft-edge/webview2/reference/win32/icorewebview2environment#add_newbrowserversionavailable).

Reiniciar é uma finalidade separada do fechamento protegido. Notas pendentes são salvas, formulários/mídia/downloads são revisados e a sessão é persistida antes de reabrir. Cancelar ou falhar no reinício mantém o navegador existente aberto. A reabertura executa diretamente o mesmo programa, sem shell, e aguarda o processo anterior sair antes de abrir o mesmo perfil.

Os textos da interface e a documentação em inglês e português permanecem sincronizados. O armazenamento do conteúdo das abas pode, ainda, ficar segregado por workspace como opt-in explícito (consulte [isolamento por workspace](../features/workspace-isolation.md)). A contenção completa do host no nível do sistema continua planejada no [desenho de isolamento AppContainer](app-container-isolation.md). Consulte também a proposta separada de [proteções futuras por reputação](../roadmap/security-protections.md).
