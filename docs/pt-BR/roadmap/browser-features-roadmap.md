# Roadmap de recursos pendentes do Veloce Browser

Este roadmap organiza os 36 recursos das Partes 1 e 2 de [recursos pendentes do navegador](../features/browser-missing-features.md) em etapas de trabalho. A ordem proposta agrupa recursos relacionados e suas dependências; não representa um cronograma nem confirma que cada recurso esteja totalmente ausente.

## Princípios de execução

- Antes de implementar cada recurso, conferir o que já existe e corrigir lacunas sem duplicar infraestrutura.
- Manter Windows e Linux no escopo. Validar em ambas as plataformas os recursos que dependem de WebViews ou integração com o sistema.
- Preservar os requisitos de segurança, privacidade e acessibilidade do projeto.
- Resolver detalhes técnicos durante o trabalho. Trazer para decisão qualquer limitação que exija mudar o escopo ou uma escolha de produto.
- Não reduzir silenciosamente a cobertura de um recurso quando uma API ou plataforma não atender ao escopo esperado.

## Etapa 0 — Levantamento e validação de escopo

Revisar os 36 itens para registrar o estado atual, os pontos de entrada existentes, as dependências e a viabilidade por plataforma. Cada item deve sair desta etapa classificado como já atendido, parcialmente atendido, pendente de implementação ou dependente de investigação técnica. Para os itens da Parte 1, registrar também as decisões de produto que precisam ser fechadas antes da implementação.

## Etapa 1 — Busca e organização de abas

- Atalhos de busca por palavra-chave: verificar o fluxo existente entre armazenamento, comandos e configurações; corrigir apenas lacunas confirmadas.
- Pesquisa rápida de abas: filtrar as abas disponíveis e permitir selecionar resultados pelo teclado, respeitando workspaces e privacidade.
- Seleção múltipla de abas: selecionar com Ctrl/Shift e aplicar ações compatíveis às abas do workspace atual.
- Rolagem da lista de abas: melhorar a rolagem no layout existente, sem presumir orientação horizontal.

**Verificação da etapa:** conflitos de atalhos, seleção por teclado, limites do workspace, proteções e confirmações das ações em lote, além de rolagem nos layouts suportados.

## Etapa 2 — Ações rápidas e utilitários locais

- Calculadora e conversões rápidas: começar com aritmética e unidades locais comuns; não usar `eval` nem serviço remoto.
- QR Code da página: gerar localmente para a URL atual e oferecer copiar ou salvar.
- Ações rápidas na barra: acrescentar ações à interface existente, mantendo as confirmações exigidas por ações destrutivas.

**Decisões antes da implementação:** definir as unidades iniciais da calculadora e quais ações serão expostas na barra. A seleção deve considerar os recursos existentes e as demais etapas deste roadmap.

**Verificação da etapa:** entradas inválidas ou não representáveis, acessibilidade por teclado e comportamento sem rede.

## Etapa 3 — Hibernação, mídia e downloads

- Exceções de hibernação: permitir exceções configuráveis por domínio, preservando a hibernação manual.
- Controle global de mídia: ampliar a visão existente para mídia em outras abas e quadros, conforme suporte das plataformas.
- Escolha do destino a cada download: integrar a preferência ao fluxo existente e tratar cancelamento, colisões e downloads nativos.

**Decisões antes da implementação:** confirmar a correspondência das exceções entre domínio e subdomínios e o estado inicial da preferência de destino dos downloads.

**Verificação da etapa:** normalização de domínios, mídia fora da aba ativa e fluxos de download nas duas plataformas.

## Etapa 4 — Recursos dependentes dos motores e das WebViews

- Captura da página inteira: investigar APIs para páginas longas, conteúdo fixo e rolagem; `CapturePreview` isoladamente não comprova cobertura integral.
- Modo escuro forçado em páginas: oferecer opção desligada por padrão e exceção por site; avaliar imagens, vídeos e páginas já escuras.
- Navegação por cursor de texto: implementada com F7 e suporte nativo do WebView2/WebKitGTK; estado temporário compartilhado entre abas da sessão. A validação nativa fica pendente para execução posterior. Ver [documentação](../features/caret-browsing.md).
- Tamanho mínimo de fonte: investigação documental concluída; WebKitGTK expõe configuração nativa e a interface documentada do WebView2 não lista equivalente. A prova visual Windows compila, mas a sessão WebDriver não inicia; validação Linux pendente. Não usar `min-font-size` como propriedade CSS. Ver [investigação](../features/minimum-font-size-investigation.md).

**Gate de viabilidade:** fazer a investigação em Windows e Linux antes de implementar ou reduzir cobertura. Se o escopo proposto não for viável, apresentar as limitações e alternativas para decisão antes de prosseguir.

## Etapa 5 — Experiência offline e personalização local ( deixar pra depois )

- Mini-jogo offline: mostrar um jogo local simples após identificar falha de conexão, sem rede ou coleta de dados, preservando ações de recuperação.
- CSS da interface do usuário: permitir um arquivo local opcional e definir um mecanismo de recuperação caso o estilo torne a interface inutilizável.

**Verificação da etapa:** distinguir falha de conexão de outros erros, manter recarregamento e navegação de recuperação acessíveis e comprovar que a interface pode ser restaurada quando o CSS falhar.

## Etapa 6 — Identidade, credenciais e dados pessoais

- Contêineres persistentes de contas: definir o isolamento entre identidades e o comportamento entre abas, workspaces e reinicializações; comprovar viabilidade por plataforma.
- Gerenciador de senhas: projetar o cofre, o desbloqueio, o preenchimento e a recuperação; submeter o modelo de segurança a auditoria antes de armazenar credenciais reais.
- Sincronização multidispositivo: escolher arquitetura e serviço, dados sincronizados, custos e requisitos de criptografia. ( nao fazer )
- Assistente de IA: definir processamento local ou externo, provedores, custos, chaves e consentimento para envio de conteúdo. ( nao fazer )

**Gate de decisão:** fechar escopo de identidade, dados e segurança antes de escolher formatos persistentes, sincronização ou integrações externas. A implementação do cofre depende do desenho e da auditoria de segurança.

## Etapa 7 — Privacidade de rede e navegação segura

- Navegação via Tor: definir objetivo de privacidade e isolamento por janela ou sessão; comprovar prevenção de vazamentos antes de qualquer promessa de anonimato.
- DNS sobre HTTPS: definir provedor, configuração, falhas e exceções; comprovar cobertura do tráfego WebView nas duas plataformas.
- Modo somente HTTPS: definir padrão, avisos, exceções e retorno a HTTP; validar detecção e cobertura das WebViews.

**Gate de decisão:** confirmar o nível de proteção e o comportamento em falhas antes de ativar qualquer proteção de rede por padrão. Verificar interação entre roteamento, resolução DNS e tráfego das WebViews.

## Etapa 8 — Proteções contra rastreamento e conteúdo malicioso

- Bloqueador avançado por regras: definir proteção, listas, atualização e tolerância a diferenças entre plataformas; avaliar a integração técnica após essas escolhas.
- Remoção de parâmetros de rastreamento: definir ativação padrão, navegações cobertas e exceções para preservar links e autenticação.
- Alerta de homógrafos e typosquatting: definir nível de proteção, experiência de aviso e tolerância a falsos positivos.
- Global Privacy Control (GPC): definir ativação padrão, exceções por site e comportamento em plataformas com cobertura limitada.
- Bloqueio de criptomineração: definir fontes e atualização das listas, cobertura e exceções; não presumir que o bloqueio por hosts cubra todos os scripts.

**Verificação da etapa:** documentar cobertura e limitações, testar exceções e evitar bloqueios ou avisos inesperados em fluxos legítimos. Ver [proteções contra rastreamento](../features/tracking-protection.md) para a implementação e pendências.

## Etapa 9 — Recursos de WebView, extensões e idiomas

- Picture-in-Picture avançado próprio: avaliar as lacunas da infraestrutura existente e definir quais capacidades adicionais justificam a solução própria e seu comportamento quando a WebView não as oferecer.
- Instalação de extensões pela Chrome Web Store: definir compromisso de compatibilidade, permissões e segurança; comprovar a viabilidade e o comportamento no Linux, cujo motor é diferente.
- Tradução de páginas: escolher processamento local, serviço externo ou ambos; definir idiomas, downloads, custos e consentimento para enviar texto.

**Gate de decisão:** confirmar cobertura e modelo de segurança antes de prometer compatibilidade de extensões ou enviar conteúdo a serviços externos. Validar as diferenças de motor e WebView nas duas plataformas.

## Etapa 10 — Organização, feeds e integração com o sistema

- Grupos de abas: definir relação com workspaces, comportamento ao mover e restaurar abas e persistência entre sessões antes de definir o modelo de dados.
- Leitor RSS: definir cadastro de feeds, frequência de atualização, retenção, privacidade e tratamento de conteúdo remoto.
- Flags experimentais: definir quem pode ativá-las, exposição em versões estáveis e limites de suporte e segurança.
- Abertura automática por extensão de arquivo: definir consentimento e tipos permitidos; restringir executáveis e arquivos potencialmente perigosos.
- Atalhos na barra de tarefas do SO: definir a experiência Linux e aceitar ou não diferenças em relação às Jump Lists do Windows.

**Verificação da etapa:** validar persistência e restauração de grupos, atualização e conteúdo remoto dos feeds, isolamento de flags experimentais, confirmação de abertura de arquivos e comportamento por sistema operacional.

## Critérios de conclusão do roadmap

- Cada um dos 36 recursos foi comparado com a implementação existente e recebeu um resultado verificável, inclusive quando a conclusão for que já está atendido.
- As decisões de produto da Parte 1 foram fechadas antes do início de cada implementação correspondente.
- Os fluxos implementados respeitam workspaces, privacidade, proteções existentes e confirmações necessárias.
- Recursos dependentes de plataforma foram avaliados em Windows e Linux; limitações e diferenças relevantes foram documentadas e trazidas para decisão.
- A documentação de cada recurso descreve o comportamento entregue e suas limitações conhecidas.
