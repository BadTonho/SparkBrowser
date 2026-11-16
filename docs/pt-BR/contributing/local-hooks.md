# Verificações locais antes do push

[English](../../en/contributing/local-hooks.md)

`npm install` e `npm ci` instalam o hook gerenciado `pre-push` pelo ciclo `prepare` do npm. Checkouts existentes podem executar `npm run hooks:install`. A instalação ignora CI e diretórios sem Git e preserva configurações personalizadas de `core.hooksPath` e hooks existentes não gerenciados. Nenhuma dependência adicional é necessária. Há suporte a Git for Windows (incluindo Git Bash) e Linux; o Node precisa estar disponível para o cliente Git.

## Pushes comuns

O hook executa `npm run validate:quick`: todos os testes JavaScript, incluindo contratos de arquitetura, e depois as verificações Svelte/TypeScript. Uma falha bloqueia o push e informa o comando que falhou. As verificações são locais e não consomem minutos do GitHub Actions. Execute também `npm run validate` ao mudar Rust ou comportamento compartilhado e os cenários nativos afetados ao mudar interface ou integração nativa.

O commit de cada referência enviada precisa corresponder ao `HEAD` atual; envie branches antigas separadamente depois de fazer checkout delas. Múltiplas referências são validadas uma única vez. Exclusões de referências e pushes sem atualizações dispensam verificações. As verificações comuns testam a árvore de trabalho: faça commit das alterações pretendidas antes do push para que os arquivos testados correspondam ao commit enviado.

## Tags de release

Quando qualquer referência de destino começa com `refs/tags/v`, o hook exige também checkout limpo, uma tag válida `vX.Y.Z` ou `vX.Y.Z-beta.N` apontando para `HEAD` e versões iguais no npm, lockfiles, Cargo e Tauri. Executa a validação completa, compila o aplicativo nativo isolado com o perfil de release e roda os sete cenários nativos críticos. A falta de dependências, drivers ou desktop utilizável bloqueia o push; resultados nativos anteriores não são reutilizados. Mudanças em `HEAD` ou na árvore de trabalho durante a validação também bloqueiam o envio.

Prepare o cache de dependências Rust e as [ferramentas nativas](native-tests.md) e mantenha o desktop de teste disponível. Use `npm run validate -- --critical` para executar as mesmas verificações antes de criar a tag. O comando local de empacotamento continua executando a suíte nativa completa. O workflow de release do GitHub faz suas próprias verificações antes de empacotar e cria um rascunho; ele não consegue verificar se um hook local foi executado.

## Hooks personalizados e limitações

Quando a instalação preservar um hook personalizado, integre `node "$root/tools/git/pre-push.mjs" "$@"` nesse hook, resolvendo `root` por `git rev-parse --show-toplevel`. Encaminhe toda a entrada padrão de quatro colunas do Git e propague códigos de saída diferentes de zero. Se várias verificações consomem a entrada, capture-a uma vez e dê uma cópia a cada verificação; apenas acrescentar o comando após outro consumidor da entrada não basta. Execute a integração a partir da raiz do repositório.

Para um `core.hooksPath` personalizado sem hook pre-push existente, copie `.githooks/pre-push` do repositório para esse diretório e dê permissão de execução ao destino no Linux com `chmod +x <diretório-de-hooks>/pre-push`. O instalador nunca substitui sua configuração automaticamente.

O `--no-verify` do Git, scripts de ciclo do npm desativados e alguns clientes gráficos podem ignorar as verificações locais. Depois de instalar com `--ignore-scripts`, execute `npm run hooks:install` explicitamente. Essas verificações são uma proteção local, não uma proteção de branch no servidor nem garantia de ausência de regressões. CI e workflows nativos continuam manuais; nenhuma execução nova em push, PR ou agendamento é ativada.
