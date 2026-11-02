# Controles de mídia das abas

No Windows, abas que emitem áudio mostram um botão de alto-falante nos dois layouts. Clique para mutar toda a aba, incluindo frames e Web Audio, sem selecioná-la. Abas mutadas mantêm o botão visível. O ajuste acompanha a aba na navegação e hibernação durante a execução; abas novas, reabertas e restauradas na inicialização começam sem mute. Preview e motores incompatíveis não exibem um controle de mute funcional.

## Volume da página

O botão de volume junto ao endereço abre um pop-up compacto de 200 × 64 px diretamente abaixo do botão para a aba focada, inclusive na divisão. Ele contém um slider horizontal, de 0 a 200% em passos de 1%, com o percentual centralizado abaixo em texto de 12 px. O percentual é atualizado imediatamente durante o arraste ou uso das setas e acompanha o nível aplicado quando o player responde. A página continua visível sem escurecimento ou desfoque. Ajuste o slider para 100% para restaurar o nível original. O mute preserva o nível escolhido. Abas novas, reabertas e restauradas na inicialização começam em 100%; navegação, hibernação e troca de workspace mantêm os ajustes durante a execução.

O slider recebe o foco do teclado; as setas ajustam o nível. Escape devolve o foco ao botão. Clicar novamente no botão ou fora do pop-up fecha o controle e confirma o último ajuste. Tab fecha o controle e devolve o foco ao botão para continuar a navegação normal pelo teclado. Trocar a aba focada, o documento ou o workspace, ou abrir outro overlay do navegador, também fecha o controle. O redimensionamento mantém sua posição junto ao botão e dentro da janela. Fontes incompatíveis e preview exibem o slider desabilitado com seu percentual abaixo. O slider fornece seu nome e percentual às tecnologias assistivas; o percentual visual não acrescenta um anúncio duplicado.

100% preserva o nível próprio do player. Mídias HTML compatíveis podem ser amplificadas localmente com Web Audio e captura do elemento, sem gravar ou transmitir conteúdo. Players incompatíveis preservam o áudio original e limitam a amplificação a 100%; fontes sem suporte ao volume mantêm o mute nativo disponível pelo botão de alto-falante da aba. Restrições de autoplay podem exigir interação com a página. Fora de um arraste ativo, o slider acompanha o nível aplicado.

O backend Rust acompanha contextos padrão de execução, incluindo frames, e vincula operações fixas à geração da WebView e ao documento. Ajustes rápidos do slider são agrupados e documentos antigos não recebem alterações destinadas à página anterior. Uma WebView filha transparente desenha o pop-up acima dos dois painéis nativos sem ocultá-los nem criar outra janela do sistema operacional. Somente suas ações autenticadas de volume e fechamento chegam à interface principal; o IPC da aplicação continua restrito à interface principal. Nenhuma preferência de mídia é gravada no SQLite ou em backups.

Execute `--case media-volume` na suíte Windows isolada para atenuação, medição da onda amplificada, independência do volume do player, agrupamento, navegação antiga, fallback entre origens e valores iniciais após reiniciar.

Players com controles HTML nativos mantêm o ajuste de 0 a 100%; a amplificação fica indisponível para preservar seu controle próprio de mute. Alterações diretas de volume pelo player mantêm o multiplicador da aba.

## Vídeo flutuante

O pop-up de volume não contém controles de vídeo. Os serviços nativos de Picture-in-Picture continuam disponíveis para os comandos internos de mídia e a suíte de regressão nativa. Somente o vídeo original selecionado entra na janela nativa Picture-in-Picture, sempre por cima; o ponto de reprodução e os controles da página continuam associados à aba original. Apenas um vídeo flutuante fica ativo por vez. Players protegidos/desativados e restrições de política em frames não são contornados.

Selecionar a aba de origem também seleciona seu workspace. O comando interno de saída fecha somente a janela flutuante. Trocar abas/workspaces e abrir overlays do navegador mantém a janela. Fechar sua aba ou navegar para outro documento encerra o modo. Um vídeo flutuante pausado ainda protege sua WebView contra hibernação e fechamento em lote pelo organizador; o diagnóstico explica essa proteção. A revisão de saída/atualização também considera o vídeo flutuante.

Notificações nativas de estado são vinculadas ao contexto de execução e documento atuais, incluindo frames de outra origem acessíveis pelo motor. A bridge de mídia informa apenas reprodução/PiP e não concede comandos do navegador a páginas externas. Motores incompatíveis e preview não podem usar os serviços nativos de mídia.

Execute `--case media-pip` para seleção de janela flutuante nativa, overlays, proteção pausada, retorno/saída, navegação, candidatos antigos, frames de outra origem e fechamento da aba de origem. Todos os cenários usam mídia gerada e perfis isolados.
