# Recursos pendentes do Spark Browser: decisões

Esta lista separa os recursos conforme a necessidade de uma decisão **sua antes da implementação**. Uma decisão técnica de rotina, investigação de viabilidade ou teste não exige necessariamente sua intervenção. A classificação não autoriza a implementação: ela serve para planejar a conversa e o trabalho.

- **Parte 1:** há escolhas de produto, privacidade, segurança ou suporte que precisam da sua orientação. As opções apresentadas não são exaustivas.
- **Parte 2:** é possível avançar com o escopo e os padrões propostos, tomando decisões técnicas durante o trabalho. Se a investigação revelar uma limitação ou mudança relevante no escopo, a decisão deve ser trazida de volta para você.

Windows e Linux continuam no escopo do projeto. Os itens que dependem de WebViews ou integração com o sistema precisam de validação em ambas as plataformas; estar na parte 2 não garante viabilidade ou prazo. A lista descreve propostas, não uma confirmação de ausência total no produto. Estimativas anteriores foram removidas por falta de validação.

## Parte 1 — Precisam da sua decisão

| Recurso | Decisão a tomar antes de implementar |
| --- | --- |
| Sincronização multidispositivo | Usar serviço central, arquitetura descentralizada ou serviço de terceiros; definir quais dados sincronizar, custos e requisitos de criptografia. |
| Bloqueador avançado por regras | Definir o nível de proteção desejado, listas e atualizações, e se limitações ou diferenças entre plataformas são aceitáveis. A integração técnica deve ser avaliada depois. |
| Contêineres persistentes de contas | Definir o isolamento esperado entre identidades e o comportamento entre abas, workspaces e reinicializações. A viabilidade por plataforma deve ser comprovada. |
| Gerenciador de senhas | Definir escopo do cofre, desbloqueio por senha mestra ou recursos do SO, preenchimento e recuperação. O modelo de segurança precisa ser projetado e auditado. |
| Assistente de IA | Escolher processamento local, serviços externos ou ambos; definir provedores, custos, uso de chaves e consentimento para envio do conteúdo. |
| Navegação via Tor | Definir o objetivo de privacidade, escopo por janela ou sessão e limites aceitáveis. A viabilidade de isolamento e prevenção de vazamentos deve ser comprovada antes de prometer anonimato. |
| DNS sobre HTTPS | Definir provedor, configuração, comportamento em caso de falha e exceções. É necessário verificar se o tráfego das WebViews pode ser coberto nas duas plataformas. |
| Picture-in-Picture avançado próprio | Definir quais recursos adicionais justificam uma solução própria e o comportamento quando a WebView não os oferecer. Já existe infraestrutura de PiP; avaliar suas lacunas primeiro. |
| Instalação de extensões pela Chrome Web Store | Definir o compromisso de compatibilidade e o comportamento no Linux, onde o motor é diferente, além do tratamento de permissões e segurança. A viabilidade da integração com a loja não está demonstrada. |
| Tradução de páginas | Escolher processamento local, serviço externo ou ambos; definir idiomas, downloads, custos e consentimento para envio de texto. |
| Flags experimentais | Definir quem pode ativá-las, se serão expostas em versões estáveis e quais limites de suporte e segurança terão. |
| Grupos de abas | Definir se grupos pertencem a um workspace, como se comportam ao mover e restaurar abas e se devem persistir entre sessões. O modelo de dados vem depois dessas escolhas. |
| Remoção de parâmetros de rastreamento | Decidir se fica ativa por padrão, em quais navegações se aplica e como funcionam exceções, pois links e fluxos de autenticação podem depender desses parâmetros. |
| Modo somente HTTPS | Definir ativação padrão, aviso e possibilidade de exceção ou retorno a HTTP. A cobertura e a detecção de falhas nas WebViews exigem investigação. |
| Alerta de homógrafos e typosquatting | Definir o nível de proteção e a experiência de alerta, incluindo tolerância a falsos positivos; uma simples checagem de alfabetos mistos não cobre todos os casos. |
| Global Privacy Control (GPC) | Definir ativação padrão e exceções por site, além do comportamento quando uma plataforma não conseguir enviar o sinal com a cobertura esperada. |
| Bloqueio de criptomineração | Definir fontes e política de atualização das listas, cobertura pretendida e exceções. O bloqueio atual por hosts não garante bloqueio de todos os scripts. |
| Leitor RSS | Definir escopo do produto: cadastro de feeds, frequência de atualização, retenção, privacidade e tratamento de conteúdo remoto. |
| Abertura automática por extensão | Definir consentimento, tipos permitidos e restrições para arquivos executáveis ou potencialmente perigosos antes de abrir downloads automaticamente. |
| Atalhos na barra de tarefas do SO | Decidir a experiência esperada no Linux e se diferenças de capacidade em relação às Jump Lists do Windows são aceitáveis. |

## Parte 2 — Podem avançar sem decisão prévia sua

Os padrões abaixo delimitam uma primeira versão; não são promessas de cobertura completa. A equipe deve respeitar os padrões de segurança e acessibilidade do projeto e validar as integrações necessárias.

| Recurso | Escopo inicial e pontos de verificação |
| --- | --- |
| Atalhos de busca por palavra-chave | Já existem tabela `site_search_shortcuts`, comandos e tela de configuração. Verificar o fluxo completo e corrigir somente lacunas identificadas, sem criar uma segunda tabela. |
| Pesquisa rápida de abas | Filtrar abas disponíveis na interface existente e selecionar resultado com teclado; respeitar as regras atuais de workspaces e privacidade. Verificar conflito do atalho. |
| Ações rápidas na barra | Acrescentar ações à interface existente; manter as confirmações já exigidas por ações destrutivas. |
| Calculadora e conversões rápidas | Começar com aritmética e unidades locais comuns, sem `eval` nem serviço remoto; definir unidades específicas durante a implementação. |
| Seleção múltipla de abas | Usar Ctrl/Shift e aplicar ações compatíveis no workspace atual; manter proteções e confirmações existentes. |
| Rolagem da lista de abas | Melhorar a rolagem do layout existente; não presumir uma barra horizontal. Botões de seta só se fizerem sentido no layout atual. |
| Exceções de hibernação | Aplicar exceções configuráveis por domínio à hibernação automática existente; preservar o comando manual e verificar normalização e subdomínios. |
| QR Code da página | Gerar localmente um código para a URL atual, com opção de copiar ou salvar; tratar entradas que não possam ser representadas. |
| Captura da página inteira | Investigar primeiro as APIs no Windows e Linux, incluindo páginas longas, conteúdo fixo e rolagem. `CapturePreview` isoladamente não comprova captura integral. Se a cobertura pretendida não for viável, apresentar as alternativas antes de reduzir o escopo. |
| Controle global de mídia | Já existem comandos e painel de mídia. Avaliar o que falta para uma visão global e reutilizar a infraestrutura; verificar mídia em outras abas e quadros nas duas plataformas. |
| Escolher destino a cada download | Acrescentar preferência ao fluxo de downloads existente, cobrindo cancelamento, colisões e downloads nativos em ambas as plataformas. |
| Mini-jogo offline | Exibir jogo local simples quando uma falha de conexão for identificada, sem rede ou coleta de dados; preservar acesso às ações de recuperação. |
| CSS da interface do usuário | Permitir arquivo local opcional para a interface, com mecanismo de recuperação caso o estilo torne a interface inutilizável. |
| Modo escuro forçado em páginas | Oferecer opção desligada por padrão e exceção por site; testar imagens, vídeos e páginas já escuras. Se os resultados forem insuficientes, relatar a limitação. |
| Navegação por cursor de texto | Implementada com suporte nativo do WebView2/WebKitGTK e F7; temporária, compartilhada entre abas de conteúdo e desligada ao iniciar. A validação nativa pode ser feita pelo cenário documentado `caret-browsing`. |
| Tamanho mínimo de fonte | Investigar suporte dos motores e alternativas com testes de acessibilidade e layout. `min-font-size` não é uma propriedade CSS padrão; não usar a regra originalmente proposta. |

## Referências no projeto

- [Atalhos de busca: repositório SQLite](../../../src-tauri/src/database/site_search_repo.rs) e [configurações](../../../src/lib/components/settings/SiteSearchSettings.svelte).
- [Comandos de mídia](../../../src-tauri/src/commands/media.rs) e [painel de mídia](../../../src/lib/components/media/MediaPanel.svelte).
- [Lista de abas existente](../../../src/lib/components/tabs/TabList.svelte).

Para as capacidades das plataformas, consultar a documentação oficial atual durante a implementação; por exemplo, [`CapturePreview` do WebView2](https://learn.microsoft.com/en-us/microsoft-edge/webview2/reference/win32/icorewebview2) e a [Media Session API](https://developer.mozilla.org/en-US/docs/Web/API/MediaSession).
