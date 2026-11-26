# Baixar vídeo: limitações em aberto

[English](../../en/roadmap/video-download-limitations.md)

Pontos em aberto do [Baixar vídeo](../features/video-download.md). Remova um item quando ele for resolvido.

## Streams com áudio separado não são suportados

**Situação:** em aberto, deixado de fora da primeira versão de propósito.

Alguns streams HLS entregam a imagem e o som em playlists diferentes (`#EXT-X-MEDIA:TYPE=AUDIO` com endereço próprio, indicado pelo atributo `AUDIO` de cada qualidade). O player junta os dois enquanto toca. O Spark só une os segmentos de uma playlist, então salvar uma qualidade assim geraria um vídeo **sem som**. Por isso essas qualidades são marcadas como `video.separate_audio` e não podem ser baixadas.

**O que o usuário vê:** observado no Reddit (`v.redd.it`). Quando o mesmo vídeo também tem uma opção que dá para salvar, como um arquivo direto, só essa opção aparece. Quando não tem nenhuma, o cartão mostra uma nota explicando o motivo. O arquivo único pode ter qualidade menor que a do stream bloqueado, e nada no painel avisa isso.

**O que é preciso para suportar:**

1. Baixar a playlist de áudio da qualidade escolhida, além da playlist de vídeo.
2. Juntar as duas faixas em um arquivo reproduzível, com um código de junção escrito em Rust (MP4 fragmentado é o caso mais simples; MPEG-TS dá mais trabalho) ou com um programa externo como o ffmpeg, que teria de ser distribuído ou baixado e mantido atualizado.
3. Verificar se esses streams também usam intervalos de bytes (`#EXT-X-BYTERANGE`), que também não são suportados. Isso não foi verificado no Reddit.

O código que lê o grupo de áudio está em `src-tauri/src/video_download/playlist.rs`.

## Alguns vídeos ficam sem prévia

**Situação:** em aberto, causa não investigada.

No Reddit, um cartão apareceu com **Sem prévia disponível**, embora o player soubesse a duração do vídeo. As prévias funcionam na página local de teste, tanto a partir de um quadro desenhado quanto de uma imagem de capa. Um ponto provável a verificar é um vídeo cujo quadro não pode ser lido e cuja capa é informada em um lugar onde a busca não olha.
