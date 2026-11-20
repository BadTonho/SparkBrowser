# Investigação de privacidade de rede

Estado: **o modo somente HTTPS e o DNS sobre HTTPS estão implementados** (desligados por padrão, veja [Privacidade de rede](../features/network-privacy.md)). **A sessão Tor está adiada**: ela está escondida da interface e nada dela está ativo.

## Resultados

- Tor (adiado): o navegador não tem integração com processo ou roteamento Tor. O desenho escolhido usa um serviço local externo e uma sessão separada e descartável, sem retorno direto. A Microsoft documenta que a [flag `proxy-server` do WebView2](https://learn.microsoft.com/en-us/microsoft-edge/webview2/concepts/webview-features-flags) afeta requisições HTTP e HTTPS. Ela não comprova, por si só, cobertura de WebSockets, WebRTC, DNS, downloads ou outros tráfegos. O WebKitGTK documenta [configuração de proxy](https://webkitgtk.org/reference/webkit2gtk/2.41.6/ctor.NetworkProxySettings.new.html), mas isso isoladamente não comprova roteamento nem prevenção de vazamentos. O suporte SOCKS e a [resolução remota de nomes do Tor](https://spec.torproject.org/socks-extensions) exigem validação ponta a ponta com os motores reais. O recurso só volta quando essa validação existir.
- DNS sobre HTTPS: nenhum dos motores tem configuração de DNS seguro controlada pelo aplicativo, e uma requisição HTTPS feita pelo Rust não configura o DNS das requisições da WebView. O desenho implementado é um proxy só de loopback que resolve os nomes por DoH e é entregue aos motores (argumentos de inicialização do WebView2, configurações de proxy do WebKitGTK). Ele é estrito (sem volta ao DNS comum) e chega ao provedor por endereço IP fixo, então a resolução inicial nunca usa o DNS do sistema. A mídia WebRTC e as requisições do próprio navegador ficam de fora.
- Somente HTTPS: o callback de navegação do Tauri recebe cada URL de nível superior e pode cancelá-la, o que basta para cancelar um endereço HTTP e abrir a forma HTTPS. As falhas vêm dos observadores nativos (WebView2 `NavigationCompleted`, WebKitGTK `load-failed`), correlacionadas com a troca pendente da guia por 60 segundos. A exceção de uso único fica presa à guia e ao endereço exato.

Os resultados descrevem as evidências disponíveis neste projeto, sem afirmar que os motores jamais poderão oferecer os recursos. Um recurso indisponível deve falhar de forma fechada; estes controles não autorizam alterar o DNS ou proxy do sistema operacional.

## Recursos de teste

`tests/native/network-privacy-fixture.mjs` inicia um serviço HTTP e um registrador de solicitações SOCKS5 somente em loopback. O endpoint SOCKS recusa as requisições e nunca encaminha tráfego; ele não é Tor. `tests/network-privacy-fixture.test.mjs` testa a fixture. O cenário nativo `network-privacy-baseline` verifica apenas navegação direta comum: sua aprovação não comprova privacidade do DNS, modo somente HTTPS ou ausência de vazamentos.

Em 7 de novembro de 2026, a linha de base do Windows passou com o WebView2 157.0.4341.42, observando um carregamento HTTP local e um redirecionamento. A linha de base do Linux ainda não foi executada.

Os testes unitários em Rust cobrem a política do modo somente HTTPS (`privacy/https_only.rs`), a leitura das respostas de DNS, o proxy (túneis, requisições comuns, falhas e requisições malformadas, contra servidores locais) e a configuração salva (`network/`).

## Validação restante

- Somente HTTPS: rodar nos dois motores com redirecionamentos reais, subquadros, navegação POST, guias restauradas e janelas abertas por páginas, e confirmar que nenhuma requisição HTTP sai antes da confirmação do usuário. O gancho de falha do WebKitGTK e o do WebView2 estão cobertos por revisão de código e testes unitários da política, ainda não por um cenário nativo.
- DNS sobre HTTPS: capturar o tráfego nos dois motores e confirmar que o tráfego das páginas chega ao proxy e que nenhum nome é resolvido pelo DNS do sistema enquanto ele está ligado, inclusive com WebSockets, workers, downloads e extensões. Confirmar que o proxy por WebView do Linux também cobre WebViews de perfis isolados. Registrar as versões dos motores.
- Tor (quando retomado): criar adaptadores nativos para cada motor; capturar o tráfego local direto e por proxy e o DNS em documentos, redirecionamentos, quadros, WebSockets, WebRTC, workers e downloads. Testar perda de conexão, reinícios, isolamento da sessão, armazenamento descartado e regras de bypass do proxy. Resolver nomes pelo Tor, não por DoH.
- Um ambiente de teste ausente é uma validação pendente, não prova de suporte nem de falta de suporte.
