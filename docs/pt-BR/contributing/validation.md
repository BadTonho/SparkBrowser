# Validação

[English](../../en/contributing/validation.md)

## Verificações automatizadas

Execute a partir da raiz do repositório:

`npm run validate:quick` executa testes JavaScript (incluindo arquitetura) e verificações Svelte/TypeScript. Execute `npm run validate -- --critical` para a validação completa com novos testes nativos críticos.

`npm run validate` executa testes JavaScript, verificações Svelte/TypeScript, rustfmt, Clippy, testes Rust, build do frontend e verificação de segurança do repositório, parando na primeira falha. `npm run validate -- --native` também compila o aplicativo nativo isolado e executa toda a suíte WebDriver. Configure as [ferramentas nativas](native-tests.md) antes; as dependências Rust precisam estar em cache para as etapas offline. Auditorias e verificações isoladas de assinaturas de instaladores continuam separadas.

```sh
npm ci
npm test
npm run check
npm run build
cargo test --manifest-path src-tauri/Cargo.toml --offline
cargo clippy --manifest-path src-tauri/Cargo.toml --offline --all-targets -- -D warnings
cargo fmt --manifest-path src-tauri/Cargo.toml --check
npm audit
npm run security:check
cargo audit --no-fetch --file src-tauri/Cargo.lock
```

No PowerShell do Windows, use `npm.cmd` quando a política de execução bloquear `npm.ps1`. As verificações Rust offline exigem dependências previamente baixadas. A cobertura da auditoria e os avisos pendentes estão descritos nas [notas de dependências](../security/dependency-notes.md).

Os testes de regressão cobrem rollback de sessão após erro de inserção e participação em transação externa; seleção de abas/workspaces; buscador escolhido; correspondência de hosts, entradas URL/Request, reutilização de XHR e beacons; correção da dependência local; e transferências HTTP reais locais, respostas de erro ou interrompidas, política de redirecionamento e colisões de nomes. Os testes de transferência usam HTTP em loopback e diretórios temporários, preservando a pasta Downloads do usuário.

As regressões de otimização também injetam falhas de sessão/ativação em checkpoints de abas, limitam o payload de listas de notas, verificam migração/atualização/exclusão/rollback dos trigramas e correspondência literal Unicode, liberam documentos de backup selecionados, comparam impressões calculadas por streaming e suprimem revisões de mídia sem mudanças. Os testes do cache de notas preservam rascunhos ativos e com falha enquanto removem entradas salvas antigas.

## Verificações nativas

A suíte WebDriver local de release isolada está documentada em [integração nativa](native-tests.md). As verificações mais amplas abaixo complementam essa cobertura automatizada.

Estas verificações exigem uma área de trabalho interativa e são separadas dos testes e builds:

1. Abra uma página com campo editável, digite um texto, abra outra aba e retorne. O texto e o histórico de voltar/avançar devem permanecer independentes.
2. Hiberne a primeira aba enquanto ela estiver inativa e depois a selecione. Somente essa página deve recarregar. Fechar outra aba em segundo plano não deve despertar páginas hibernadas.
3. Selecione pela Command Bar uma aba de outro workspace. O botão do workspace, o tema e a lista de abas devem concordar. Mover a aba ativa para outro workspace deve preservar essa consistência.
4. Escolha DuckDuckGo e pesquise pelas duas ações da Command Bar: aba atual e nova aba. Ambas devem usar DuckDuckGo.
5. Baixe um recurso HTTP direto. Confira os bytes no disco, nomes sem sobrescrita e erro para uma resposta inválida. O preview no navegador comum não deve fabricar downloads concluídos.
6. Altere a proteção no Windows e confira a preservação dos formulários e do histórico de voltar/avançar. Teste requisições nos documentos atuais e após navegar. Em outros motores, verifique o fallback de recarregamento documentado e a recusa enquanto formulários/mídia/downloads detectados estiverem protegidos.
7. Navegue para um host bloqueado. A última URL confirmada deve permanecer. Uma navegação comum concluída deve adicionar uma entrada ao histórico.

Finalize conferindo `git status`, `git diff`, `git diff --cached` e as regras de ignore para arquivos sensíveis.

## Arquitetura e contratos entre funcionalidades

`npm run architecture:check` está incluído pelos testes JavaScript na validação consolidada. Execute `npm run test:native:critical` após compilar o aplicativo nativo isolado para conferir os cenários obrigatórios entre funcionalidades. CI e workflows nativos são manuais. O workflow de release verifica as duas plataformas antes de empacotar, mas não executa cenários nativos. Consulte o [fluxo de regressões](regression-workflow.md) e a [responsabilidade pelo estado](../architecture/state-ownership.md).


## Dados de teste do cofre de senhas

Só em builds de desenvolvimento (`npm run tauri dev`), Configurações → Senhas mostra um cartão tracejado **Só para testes** com **Adicionar senhas de teste** e **Remover senhas de teste**. O primeiro adiciona uma dúzia de logins inventados (várias contas em um site, um endereço com `www.`, um login sem site, um título comprido) para testar a lista com dados; apertar duas vezes não adiciona nada em dobro. O segundo remove apenas as entradas cujas notas começam com `[dados de teste]`, nunca algo que você salvou. Os botões não existem em builds de release: a interface verifica `import.meta.env.DEV` e os comandos do backend só existem com `debug_assertions` ligado (senão respondem `vault.dev_only`). O cofre precisa estar desbloqueado.


## As telas de Configurações são projetadas

A aba de Configurações não executa os componentes de Configurações: ela mostra uma projeção da interface (veja `src/lib/services/panel-presentation.ts`). Só uma lista de tags e atributos atravessa, e só botões, formulários e campos podem ser clicados lá. Uma tag fora da lista (por exemplo `dialog`, `fieldset`, `legend`, `a` ou `rect` do SVG), um `style` inline, ou uma contenção de CSS (`container-type`, `contain`) que prenderia sobreposições `position: fixed` a uma seção simplesmente não funciona na aba, embora pareça certo em um navegador comum. Monte modais com elementos simples e um botão como fundo, use classes para cores e fique nas tags permitidas. O `tests/password-vault.test.mjs` confere isso nas telas de senhas.
