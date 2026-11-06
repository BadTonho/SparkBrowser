# Aparência da barra e do conteúdo

Abra **Configurações → Aparência e disposição → Barra e conteúdo** para ajustar a barra de endereços e a moldura principal da WebView. As alterações são aplicadas na hora e salvas localmente para a próxima inicialização.

- **Usar barra de endereços larga** remove o limite de 980 px para que a barra use o espaço livre entre os controles. Ela ainda se ajusta para caber em janelas estreitas e na visualização dividida.
- **Sempre mostrar URLs completos** mantém `https://` e `www.` visíveis quando a barra está ociosa. Desativada, oculta esses prefixos em endereços HTTP e HTTPS; ao focar o campo, o endereço completo fica disponível para edição. A navegação continua usando a URL original.
- **Usar modo compacto** restaura o tamanho menor atual: barra com 40 px de altura e campo de endereços com 28 px. O modo normal é o padrão e usa 44 px e 32 px, com controles proporcionalmente maiores. O menu responsivo de ações em telas estreitas e na visualização dividida continua independente.
- **Mostrar cantos arredondados no conteúdo** restaura a borda, os cantos arredondados e a sombra ao redor da área principal da WebView. A opção começa desativada. O modo Zen continua sem moldura e de ponta a ponta, independentemente dessa preferência.

As preferências ficam no armazenamento local da interface e não recarregam as abas.
