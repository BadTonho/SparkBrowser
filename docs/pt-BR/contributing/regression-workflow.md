# Fluxo de prevenção de regressões

Antes de alterar um comportamento compartilhado, identifique o domínio responsável e escreva um teste de comportamento que atravesse a fronteira afetada. Separe movimentações mecânicas das mudanças de comportamento na descrição da revisão. Preserve os dados IPC, a compatibilidade dos backups, os perfis persistentes e o suporte a Windows/Linux.

## Validação local

- `npm run architecture:check` rejeita ciclos de imports estáticos de execução entre serviços/stores do frontend, imports privados do estado do navegador, imports de mutação de sessão não autorizados e acesso à fábrica de conteúdo fora do serviço responsável. A visibilidade do Rust também impede guards mutáveis de sessão fora do módulo de estado.
- `npm run validate` inclui arquitetura, contratos JavaScript, verificação Svelte, formatação Rust, Clippy estrito, testes Rust, build do frontend e verificações de segurança do repositório.
- Compile o aplicativo isolado com `npm run test:native:build:fast` e execute `npm run test:native:critical`. Use `npm run test:native` para a suíte completa. Consulte a [configuração nativa](native-tests.md).

A suíte crítica contém explicitamente `backup-identities`, `persistence-split`, `persistence-workspaces`, `workspace-switcher`, `recovery`, `shutdown` e `security-boundaries`. Cada cenário usa seu próprio perfil e fixtures locais. O cenário de backup combina containers, abas descartáveis, layout dividido, formulário residente e reinício após encerramento forçado. Os cenários de persistência injetam falhas no banco e repetem operações sem descartar documentos residentes.

Os contratos também exercitam observações fora de ordem, falha ao mover de workspace, cancelamento de arraste após fechar abas, descarte durante a inicialização, assinaturas compartilhadas e aberturas concorrentes de painéis. Os testes de proteção existentes cobrem formulários, mídia, downloads, abas fixadas e painéis visíveis. Os cenários nativos de permissão cobrem cancelamento de pedidos e decisões lembradas; não introduza uma nova política de hibernação durante uma refatoração.

## Integração contínua e releases

O workflow regular mantém a validação em Windows e Linux. Os testes nativos críticos do Windows executam em cada pull request; a suíte nativa completa executa diariamente às 06:00 UTC, manualmente e como dependência da publicação de releases. Falhas reprovam o workflow, sem novas tentativas automáticas ou exclusões silenciosas. Corrija os pré-requisitos de plataforma/driver quando falharem; não transforme cenários obrigatórios em verificações opcionais.

O runner imprime diagnósticos localmente e registra nomes, resultados e durações em `.cache/native-results.json`. A CI publica somente esse resumo revisado, por sete dias. Perfis, bancos, logs brutos e capturas ficam fora dos artefatos publicados. Os artefatos locais dos cenários isolados permanecem em `.cache/native-runs/`, conforme a política de limpeza existente.

O caminho do driver nativo Linux foi preservado, mas não foi validado neste ambiente Windows. Foco da área de trabalho, captura de ponteiro, prompts reais de dispositivos/SO e contas externas autenticadas mantêm as limitações descritas na documentação dos testes nativos. A definição de um workflow não comprova aprovação em um runner hospedado.

Antes de entregar, examine `git status`, diffs de trabalho e staged, regras de ignore e verificações de segredos. Toda documentação nova deve ter caminhos correspondentes em inglês e pt-BR. Registre na revisão as verificações realmente executadas e quaisquer falhas ou limitações de plataforma.
