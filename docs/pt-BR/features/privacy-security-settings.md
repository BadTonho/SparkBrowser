# Configurações de privacidade e segurança

A categoria **Privacidade e segurança** reúne o histórico de navegação local, as permissões de sites, o comportamento de pop-ups, o escudo contra rastreadores e a proteção de navegação segura disponível.

## Histórico de navegação local

A ação **Apagar histórico de navegação local** remove o histórico de navegação do perfil do Spark e o cache de ícones dos sites recentes. Ela não apaga cookies, armazenamento dos sites, cache do navegador nem downloads.

A retenção automática do histórico começa em **3 meses**. Escolha 1, 3, 6 ou 12 meses, ou **Sempre** para manter o histórico sem expiração automática. Os registros vencidos são removidos ao registrar uma visita, quando a preferência de retenção muda e ao iniciar o navegador.

**Nunca salvar** impede o registro de novas visitas, inclusive as pendentes quando a configuração muda. O histórico anterior permanece disponível até ser apagado manualmente ou expirar por outra opção de retenção. A preferência é mantida após reiniciar.

## Navegação segura

No Windows, a configuração do Microsoft Defender SmartScreen controla a verificação de reputação do WebView2 e fica ativada por padrão quando o runtime oferece suporte. O WebView2 pode enviar URLs e informações relacionadas à Microsoft para verificar a reputação de uma página. As políticas do Windows ou da organização podem prevalecer sobre a configuração do navegador. Consulte a [documentação de privacidade de dados do WebView2](https://learn.microsoft.com/en-us/microsoft-edge/webview2/concepts/data-privacy?tabs=dotnetcsharp), da Microsoft.

Esse controle não está disponível no Linux nem em runtimes do WebView2 para Windows que não exponham a configuração necessária. O Spark não substitui nem simula essa proteção em plataformas sem suporte.

## Proteção de sites

A configuração de pop-ups controla se páginas web podem abrir destinos HTTP ou HTTPS em novas guias do Spark. Ela começa desativada para preservar o comportamento atual de bloqueio do navegador. A janela de picture-in-picture do próprio Spark é tratada separadamente e continua disponível quando pop-ups de páginas estão desativados.

O escudo contra rastreadores e as decisões salvas de permissões de sites também ficam nesta categoria. As permissões de sites abrangem os tipos de permissão expostos pela plataforma e pelo runtime do navegador atual.
