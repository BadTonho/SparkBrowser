# Navegação por cursor de texto

Pressione **F7** para ligar ou desligar a navegação por cursor de texto. Um cursor aparece no conteúdo da página: use as setas para percorrer o texto e **Shift** junto às setas para selecioná-lo. O motor da página controla o comportamento em links, quadros e campos editáveis.

O modo começa desligado sempre que o app é aberto e vale para todas as abas de conteúdo durante a sessão. Abas novas herdam o estado da sessão. Nada é gravado nas preferências ou nos backups.

No Windows, o recurso usa o atalho nativo do WebView2. No Linux, usa a configuração nativa `enable-caret-browsing` do WebKitGTK. O navegador não desenha um cursor simulado nem injeta código para selecionar texto. O movimento e o foco exatos seguem o motor WebView instalado.

Execute `npm run test:native:build:fast` e depois `npm run test:native -- --case caret-browsing` para verificar F7, seleção, links, um quadro incorporado, campos editáveis e o estado compartilhado entre abas existentes e novas.
