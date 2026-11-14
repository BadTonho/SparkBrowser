# Aceleração de hardware

Configurações → Desempenho tem um interruptor **Aceleração de hardware**. Ele vem **ligado por padrão**: páginas e vídeos são desenhados pela placa de vídeo, o que é mais rápido e usa menos CPU. Desligue se vir janelas em branco ou brancas, piscadas, rasgos de imagem ou outras falhas gráficas, que alguns drivers e máquinas virtuais causam. As páginas passam a ser desenhadas por software: tudo funciona, mas páginas pesadas e vídeos usam mais CPU.

## Aplicando uma mudança

O motor web lê as opções gráficas uma única vez, quando o processo do navegador inicia, então a mudança vale depois de reiniciar:

1. Mude o interruptor. A escolha é salva na hora.
2. Aparece um aviso (“Reinicie o Spark para aplicar esta mudança.”) com o botão **Reiniciar agora**.
3. **Reiniciar agora** usa o mesmo encerramento protegido de uma atualização do WebView2: notas pendentes e a sessão são salvas antes, e o Spark volta já com a nova configuração. Você também pode simplesmente fechar e abrir o Spark depois.

Se você voltar o interruptor antes de reiniciar, o aviso some: a configuração é comparada com o que o processo em execução realmente usou ao iniciar.

## Como funciona

- A escolha fica em `graphics.json` dentro da pasta de dados do app, e não no banco nem na interface, porque precisa ser lida antes de existir qualquer webview.
- No Windows, com a aceleração desligada, o Spark acrescenta `--disable-gpu` a `WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS`; argumentos que você mesmo definiu nessa variável são mantidos. No Linux, define `WEBKIT_DISABLE_COMPOSITING_MODE=1` e `WEBKIT_DISABLE_DMABUF_RENDERER=1` (WebKitGTK), a menos que você já as tenha definido.
- Janelas privadas e processos reiniciados herdam o que o processo principal configurou. Um processo reiniciado remove apenas as opções que o próprio Spark adicionou, controladas por uma marca interna (`SPARK_GRAPHICS_FLAGS`), então religar a aceleração nunca apaga algo que você definiu.
- Um `graphics.json` ausente, ilegível ou danificado significa aceleração **ligada**. Uma marca de ordem de bytes (BOM) no início do arquivo, que o Bloco de Notas e o Windows PowerShell acrescentam ao salvar em UTF-8, é ignorada, então um arquivo editado à mão funciona.

## Fora dos backups

Problemas de GPU pertencem a uma máquina e a um driver, então esta configuração não entra em backups, arquivos de sincronização nem em preferências importadas. Restaurar um backup em outro computador nunca desliga a aceleração ali.

## Limites

- O interruptor controla só o uso de GPU do webview. Configurações do sistema ou do driver (por exemplo, a escolha de GPU por aplicativo no Windows) são independentes.
- Na prévia em navegador usada no desenvolvimento não há escolha de GPU a fazer, então o interruptor mostra a aceleração como ligada e nunca pede reinício.

## Implementação

- Nativo: `src-tauri/src/graphics.rs` (arquivo, ambiente, status) e `commands/graphics.rs` (`get_graphics_status`, `set_hardware_acceleration`, `restart_for_graphics`). As opções são definidas em `run()` antes de existir o primeiro webview.
- Interface: `graphicsStore.ts` e a linha em `PerformanceSettings.svelte`.
- Testes: testes unitários em `graphics.rs` cobrem o padrão, arquivos danificados, um arquivo com marca de ordem de bytes, a combinação de argumentos e o ciclo completo do ambiente. O efeito num WebView2 real foi conferido subindo a versão de teste nativo com a opção desligada e lendo a linha de comando do processo principal do navegador, que então traz `--disable-gpu`.
