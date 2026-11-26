# Baixar vídeo

[English](../../en/features/video-download.md)

Um botão opcional da barra que salva vídeos da página carregada. Ele começa desligado: ligue em **Configurações** (a engrenagem) → **Botões da barra** → **Baixar vídeo**.

## Como usar

O botão abre um pop-up logo abaixo dele, por cima da página (que continua visível), que procura vídeos na aba ativa. Ele segue o tema, a paleta e o estilo de janela escolhidos, como os outros pop-ups. Há um cartão por vídeo, com os que estão na tela primeiro. Cada cartão tem uma pequena prévia (um quadro do vídeo, ou a imagem de capa quando o site não permite ler o quadro), a duração quando o player a conhece e a marca **Na tela**. Abaixo, cada opção mostra o tipo (arquivo de vídeo ou stream HLS), a qualidade quando conhecida, o nome do arquivo e o servidor; as que dá para salvar vêm primeiro, a melhor qualidade primeiro. Clique em **Baixar**; o progresso, o cancelamento e o arquivo salvo ficam no [painel de downloads](downloads.md). Valem as configurações **Perguntar onde salvar cada download** e a pasta de downloads. O nome sugerido vem do título da página.

Se a lista estiver vazia, dê play no vídeo e clique em **Procurar de novo**: muitos players só pedem o vídeo depois que a reprodução começa. Pressione **Esc**, clique fora ou clique de novo no botão para fechá-lo; ele também fecha quando você troca de aba ou a página muda de endereço.

Um vídeo pode ficar sem prévia quando o site não permite nenhuma das duas. Endereços apenas vistos sendo pedidos são ligados a um vídeo pela pasta; quando isso não é certo, ganham um cartão próprio.

## O que é suportado

| Origem | Suporte |
| --- | --- |
| Arquivos de vídeo diretos (MP4, WebM e semelhantes) | Sim, com pausa, retomada e verificação como qualquer download por URL |
| Streams HLS (`.m3u8`) com segmentos MPEG-TS | Sim, salvos como `.ts` (streams só de áudio como `.aac` ou `.mp3`) |
| Streams HLS com segmentos MP4 fragmentados | Sim, salvos como `.mp4` |
| YouTube e outros players que só usam DASH ou Media Source | Não |
| DRM ou qualquer HLS criptografado, inclusive AES-128 | Não |
| Transmissões ao vivo | Não |
| HLS com áudio em faixa separada, intervalos de bytes ou várias seções de inicialização | Não; o item fica desabilitado com o motivo |

Os segmentos são unidos sem conversão e nenhum programa externo é usado. Um arquivo `.ts` toca em players como o VLC. Baixe apenas conteúdo que você tem o direito de salvar.

## Limitações

- Os cookies do site não são enviados, então vídeos que exigem login podem falhar. O pedido envia um `Referer` como os navegadores: outros servidores só ficam sabendo a origem da página.
- Streams não podem ser pausados. Um stream interrompido ou pausado (por exemplo, depois de fechar o Spark) recomeça do início ao ser retomado. O tamanho total é desconhecido até o fim.
- Um stream é limitado a 10.000 segmentos, e suas playlists a 2 MB.
- Playlists pedidas dentro de Web Workers, e vídeos dentro de componentes web fechados, não são encontrados. Só 8 playlists são lidas por busca.
- No Linux, apenas o quadro principal da página é examinado, então players em quadros incorporados não são encontrados.

## Privacidade e segurança

Enquanto uma página está aberta, o Spark mantém na memória dela os endereços das últimas 32 playlists `.m3u8` pedidas; nada é salvo nem enviado. A página só é examinada quando você abre o painel: o Spark pede os endereços que ela já usa (elementos de vídeo, inclusive dentro de componentes web abertos, e pedidos `.m3u8`) e lê no máximo 8 playlists para listar as qualidades. Todo endereço, redirecionamento e segmento precisa ser HTTP(S) sem credenciais embutidas e passa pelo escudo de privacidade. Os dois comandos só estão disponíveis para a interface local.

## Código

- `src-tauri/src/media/controller.js`: a ação `sources`.
- `src-tauri/src/video_download/`: a busca e o leitor de playlists HLS.
- `src-tauri/src/downloads/hls.rs`: o download de streams.
- `src/lib/components/video-download/`: o painel e o estilo.
- `src-tauri/src/window/video_download_popover/`: o pop-up nativo.
