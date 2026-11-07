# Investigação de privacidade de rede

Estado: **sessão Tor, DNS sobre HTTPS e modo somente HTTPS estão indisponíveis**. Nenhum está ativado ou é apresentado como proteção. As configurações de privacidade mostram essas limitações; nenhuma configuração, provedor DNS, proxy ou exceção HTTP de uso único está ativo.

## Resultados

- Tor: o navegador não tem integração com processo ou roteamento Tor. O desenho escolhido usa um serviço local externo e uma sessão separada e descartável, sem retorno direto. A Microsoft documenta que a [flag `proxy-server` do WebView2](https://learn.microsoft.com/en-us/microsoft-edge/webview2/concepts/webview-features-flags) afeta requisições HTTP e HTTPS. Ela não comprova, por si só, cobertura de WebSockets, WebRTC, DNS, downloads ou outros tráfegos. O WebKitGTK documenta [configuração de proxy](https://webkitgtk.org/reference/webkit2gtk/2.41.6/ctor.NetworkProxySettings.new.html), mas isso isoladamente não comprova roteamento nem prevenção de vazamentos. O suporte SOCKS e a [resolução remota de nomes do Tor](https://spec.torproject.org/socks-extensions) exigem validação ponta a ponta com os motores reais.
- DNS sobre HTTPS: o navegador ainda não possui integração de DNS seguro controlada pelo aplicativo para as WebViews. Uma requisição HTTPS feita pelo Rust não configura o DNS das requisições da WebView. O modo estrito precisa de cobertura no motor, escolha explícita de provedor e prova de que falhas não retornam silenciosamente ao DNS comum, inclusive na resolução inicial do provedor e em redirecionamentos.
- Somente HTTPS: o callback atual de navegação do Tauri recebe uma URL e pode cancelar a navegação; o callback compartilhado não expõe todo o contexto necessário para uma exceção autenticada de uso único e a correlação de falhas. O Windows também tem observadores nativos de navegação, que hoje informam o estado sem impor a troca para HTTPS. Uma implementação completa precisa verificar quadros, redirecionamentos, POST, novas janelas, restauração e falhas sem alterar a semântica da requisição nem permitir HTTP antes da confirmação.

Os resultados descrevem as evidências disponíveis neste projeto, sem afirmar que os motores jamais poderão oferecer os recursos. O comportamento nativo em Windows e Linux ainda precisa ser medido. Um recurso indisponível deve falhar de forma fechada; estes controles não autorizam alterar o DNS ou proxy do sistema operacional.

## Recursos de teste

`tests/native/network-privacy-fixture.mjs` inicia um serviço HTTP e um registrador de solicitações SOCKS5 somente em loopback. O endpoint SOCKS recusa as requisições e nunca encaminha tráfego; ele não é Tor. `tests/network-privacy-fixture.test.mjs` testa a fixture. O cenário nativo `network-privacy-baseline` verifica apenas navegação direta comum: sua aprovação não comprova roteamento Tor, privacidade do DNS, modo somente HTTPS ou ausência de vazamentos.

Em 7 de novembro de 2026, o cenário básico passou no Windows com WebView2 157.0.4341.42, observando um carregamento HTTP local e um redirecionamento. O cenário básico Linux não foi executado.

`src-tauri/src/privacy/network_policy.rs` é compilado apenas para testes unitários. Ele explora a conversão de URL para HTTPS, permissões exatas de uso único vinculadas à aba, geração da WebView e navegação, e validação de endpoint HTTPS. Não está conectado à navegação nem à configuração de provedor e não é uma política de produção.

## Validação restante antes da ativação

- Construir adaptadores nativos por motor; capturar tráfego local direto e via proxy e DNS em documentos, redirecionamentos, quadros, WebSockets, WebRTC, workers e downloads. Testar perda da conexão, reinício, isolamento de sessão, descarte de dados e regras que ignoram o proxy. Definir como manter o bloqueio em falhas em todos os caminhos cobertos.
- Verificar a resolução inicial do endpoint DoH e falhas do provedor sem mudar o resolvedor do sistema; confirmar que todo o tráfego WebView informado segue a política escolhida. Em sessões Tor, resolver nomes pelo Tor, sem requisição DoH separada.
- Verificar se conversões para HTTPS e eventos de falha correspondem exatamente à navegação original. Testar autorização única, callbacks antigos, redirecionamentos, requisições não idempotentes, portas, fragmentos, subquadros e recuperação sem retorno silencioso a HTTP.
- Executar os cenários nativos em Windows e Linux quando cada ambiente estiver disponível e registrar as versões dos motores. A ausência de ambiente de teste é uma validação pendente, não prova de suporte nem de ausência de suporte. Manter o recurso indisponível em uma plataforma até verificar o comportamento anunciado.
