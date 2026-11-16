# Plano de fundo que reage ao áudio

[English](../../en/features/audio-background.md)

Abra **Personalizar nova guia → Plano de fundo → Equalizador de áudio** e ative **Mostrar visualizador que reage ao áudio**. O efeito começa desativado. Escolha um estilo visual, uma cor e a intensidade visual entre 0,5 e 2,0; esses controles nunca alteram o volume da reprodução.

O visualizador aparece sobre a cor, imagem ou vídeo atual, atrás dos controles da página. Os estilos em faixa ficam no quarto inferior da nova guia; os estilos em tela cheia usam a página inteira. Ele reage ao espectro real do áudio, incluindo Web Audio, das guias normais em todas as janelas e workspaces do mesmo perfil. O visual desaparece em aproximadamente meio segundo de silêncio. O modo de movimento reduzido usa atualizações discretas e menos frequentes em vez de transições suaves.

## Estilos visuais

Todos os estilos são desenhados a partir dos mesmos 32 níveis reais de frequência e usam a cor escolhida. **Barras** é o padrão.

| Grupo | Estilos |
|---|---|
| Linhas e ondas | Barras, Onda, Área / montanha, Linhas empilhadas, Linha de batimento |
| Circulares | Barras radiais, Anel pulsante, Anéis concêntricos, Espiral, Disco de vinil |
| Pontos e partículas | Pontos de LED, Partículas, Bolhas, Vaga-lumes |
| Ambiente | Brilho nas bordas, Orbes, Espectrograma, Aurora |
| Geométricos | Polígonos, Terreno 3D |

Barras, Onda, Área / montanha, Linhas empilhadas, Linha de batimento, Pontos de LED e Espectrograma usam o quarto inferior da página. Todos os outros estilos usam a página inteira. O código de desenho fica em `public/new-tab/audio-styles.js`; Barras faz parte de `audio-background.js`. Os nomes de estilo aceitos são validados pelo backend, e um nome desconhecido é rejeitado.

## Plataformas e captura

- **Windows:** requer build 20348 ou posterior e um runtime WebView2 compatível. O WASAPI Application Loopback captura apenas as árvores dos processos WebView2 normais relevantes, sem duplicação entre guias. Ele nunca monitora a área de trabalho inteira ou a árvore completa de processos do aplicativo Spark.
- **Linux:** requer `libpulse.so.0` e um servidor PulseAudio ou o servidor de compatibilidade PulseAudio do PipeWire em execução. Pacotes da família Debian recomendam `libpulse0`; pacotes RPM recomendam `pulseaudio-libs`. A biblioteca é carregada sob demanda, portanto sua ausência não impede o Spark de iniciar. Cada stream elegível é monitorado individualmente. Seu PID e a cadeia de processos WebKit precisam corresponder ao processo normal do Spark; streams ambíguos são excluídos.

O painel de configurações explica quando a captura está indisponível. Conteúdo protegido pode retornar silêncio. Não existe espectro simulado ou tentativa de contornar a proteção do conteúdo. Áudio no aplicativo Spotify, no celular ou em outro navegador não movimenta o efeito; o Spotify em uma guia do Spark pode participar quando o sistema fornece áudio capturável.

## Ciclo de vida e privacidade

Um serviço nativo compartilha a captura e a análise FFT entre as novas guias visíveis. A captura começa somente com o efeito ativado, uma nova guia visível em janela não minimizada e uma guia normal emitindo áudio com volume diferente de zero. A captura para quando essas condições deixam de existir. Novos streams e mudanças de saída de áudio são detectados e recebem novas tentativas automaticamente. Fechar, navegar, ocultar ou desativar um consumidor remove sua assinatura de renderização.

O analisador não abre microfone, grava arquivo, transmite áudio, muda o roteamento da reprodução ou ajusta o volume. PCM existe apenas em buffers nativos temporários e limitados. O documento local da nova guia recebe 32 níveis normalizados, no máximo 20 vezes por segundo, pelo bridge autenticado e vinculado ao documento. Quadros antigos ou inválidos são ignorados. O Canvas para de animar quando as barras chegam ao silêncio e limpa níveis antigos se as atualizações cessarem.

As preferências são locais do dispositivo, sincronizadas entre novas guias normais e restauradas ao reiniciar. Backups não as incluem. Janelas privadas começam com o efeito desativado e preferências padrão; suas árvores de processos de áudio não participam da captura do perfil normal.

## Validação

Testes unitários cobrem silêncio, ordenação de frequências, variações de amplitude, tons combinados, validação das preferências, layout ABI e filtro de processos Linux, ciclo de vida dos consumidores, movimento reduzido e mensagens antigas ou repetidas no bridge. Os fontes da captura Linux também compilam nos testes unitários Windows, sem carregar libpulse nesse sistema. Execute o cenário nativo isolado `audio-background` para captura real por processo, pico de 440 Hz, mute, volume zero, visibilidade, persistência e preservação da reprodução. Verificações com servidor de áudio Linux e players protegidos exigem o ambiente correspondente.
