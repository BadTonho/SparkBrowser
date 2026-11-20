# Privacidade de rede

[English](../../en/features/network-privacy.md)

O grupo **Privacidade de rede**, em **Configurações → Privacidade e segurança**, tem duas proteções. As duas vêm desligadas. Uma terceira, a sessão Tor, está [adiada](#sessão-tor-adiada) e não aparece na interface.

## Modo somente HTTPS

Com o interruptor ligado, toda navegação `http://` de nível superior é cancelada e a versão `https://` é aberta no lugar (mesmo host, porta, caminho, consulta e fragmento). Vale na hora, sem reiniciar.

- **Quando a versão segura não pode ser acessada**, aparece um aviso no topo da janela: *Conexão segura indisponível*. **Continuar por HTTP** abre o endereço original **uma vez**.
- **A exceção é exata e vale uma vez.** Ela fica presa a uma guia e àquele endereço exato. A navegação seguinte (para qualquer endereço, inclusive o mesmo) é trocada para HTTPS de novo, e outra guia nunca herda a exceção. Desligar o modo descarta todas as exceções pendentes.
- **Não são alterados:** endereços `https://` e endereços da própria máquina (`localhost`, `*.localhost`, `127.0.0.0/8` e `::1`), que não têm certificado para verificar.
- **Endereços com usuário ou senha** em `http://` nunca são abertos (não existe forma segura deles em HTTPS).
- Um host digitado sozinho na barra de endereço (`example.com`) já abre como `https://`, com ou sem o modo.

O que não cobre: recursos que a página carrega (imagens, scripts) e quadros. Os navegadores bloqueiam ou atualizam esses itens como conteúdo misto, e os motores já fazem isso em páginas carregadas por HTTPS. O modo age no endereço da própria página.

Como uma falha é reconhecida: a troca fica registrada para a guia por 60 segundos. Um erro de navegação dessa guia dentro do prazo (`NavigationCompleted` com erro no WebView2, ou `load-failed` no WebKitGTK) abre o aviso. Uma navegação cancelada pelo próprio navegador, como o endereço HTTP substituído, não é erro. Uma página que carregou corretamente limpa a troca pendente.

Código: [`privacy/https_only.rs`](../../../src-tauri/src/privacy/https_only.rs) (política e testes), o callback de navegação em `window/content_factory.rs`, os ganchos de falha em `window/navigation_state.rs` e o aviso em `HttpsOnlyNotice.svelte`.

## DNS sobre HTTPS

O WebView2 e o WebKitGTK não têm configuração de DNS seguro. Por isso o Spark inicia um pequeno **proxy no endereço de loopback** (`127.0.0.1`, um endereço que só este computador alcança) e aponta os motores web para ele. O proxy resolve cada nome de host pelo provedor escolhido usando DNS sobre HTTPS e depois conecta ao endereço obtido.

- **Provedores:** Cloudflare (`1.1.1.1`) e Google (`8.8.8.8`). O servidor do provedor é acessado por um endereço IP fixo, então perguntar pelo provedor nunca usa o DNS do sistema.
- **Modo estrito:** se o provedor não puder ser acessado, ou um nome não puder ser resolvido, a página não carrega (o proxy responde *502 Bad Gateway* com uma explicação). **Não há volta** ao DNS comum. Para navegar sem ele, escolha **Desligado** e reinicie.
- **Reiniciar aplica a mudança.** O motor recebe o endereço do proxy quando inicia, como a [aceleração de hardware](hardware-acceleration.md). A escolha é salva em `network.json` na pasta de dados, lida antes de qualquer WebView existir. A configuração mostra **Reiniciar agora** enquanto há mudança pendente. Janelas privadas usam o proxy da janela que as abriu.
- **Windows:** o proxy entra nos argumentos de inicialização do WebView2, compartilhados por todas as WebViews do processo. **Linux:** cada WebView de conteúdo recebe o proxy pelas configurações de proxy do motor.

O que cobre: o que as páginas carregam pela rede do motor (documentos, quadros, scripts, imagens, WebSockets e downloads por HTTP e HTTPS).

O que **não** cobre:

- Mídia WebRTC (UDP), que não passa por proxy HTTP.
- As requisições do próprio navegador feitas pelo aplicativo, como atualizações, ícones de sites e sugestões de busca.
- Qualquer coisa fora do Spark. O DNS e o proxy do sistema nunca são alterados.
- O provedor vê os nomes que você consulta. O DNS sobre HTTPS os esconde da rede local, não do provedor.

Notas de segurança: o proxy escuta só no endereço de loopback, recusa conexões que não venham dele e repassa o que recebe: qualquer programa deste computador pode usá-lo como resolvedor enquanto o Spark estiver aberto. Requisições `http://` comuns são repassadas uma por conexão. Uma falha de consulta é gravada no [registro de erros](error-log.md) como aviso, sem o nome do host.

Código: [`network/`](../../../src-tauri/src/network) (`mod.rs` para a configuração e a inicialização, `resolver.rs` para as consultas, `proxy.rs` para o proxy) e `commands/network.rs`.

## Sessão Tor (adiada)

A **sessão Tor** está **adiada** e não aparece nas Configurações. Nada dela está ativo. Ela exige um desenho que comprove, com os motores reais, que nenhum tráfego vaza fora do Tor (DNS, WebRTC, WebSockets, downloads, extensões) e que a sessão é isolada e descartável. Um interruptor que não consegue garantir isso seria pior do que nenhum, então ela só volta quando essa prova existir. Os resultados estão na [investigação de privacidade de rede](../security/network-privacy-investigation.md).

## Verificação

- Testes em Rust: a política do modo somente HTTPS (`privacy::https_only`), as respostas de DNS, o proxy e a configuração salva (`network::`). Os testes do proxy rodam túneis, requisições comuns e falhas contra servidores locais, sem internet.
- O cenário nativo `network-privacy-baseline` ainda verifica só a navegação direta comum. Uma verificação completa das duas proteções em cada sistema operacional é uma validação pendente: registre as versões dos motores quando ela for feita.
