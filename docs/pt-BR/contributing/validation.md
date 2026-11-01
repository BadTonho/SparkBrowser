# Validação

[English](../../en/contributing/validation.md)

## Verificações automatizadas

Execute a partir da raiz do repositório:

```sh
npm ci
npm test
npm run check
npm run build
cargo test --manifest-path src-tauri/Cargo.toml --offline
cargo clippy --manifest-path src-tauri/Cargo.toml --offline --all-targets -- -D warnings
cargo fmt --manifest-path src-tauri/Cargo.toml --check
npm audit
cargo audit --no-fetch --file src-tauri/Cargo.lock
```

No PowerShell do Windows, use `npm.cmd` quando a política de execução bloquear `npm.ps1`. As verificações Rust offline exigem dependências previamente baixadas. A cobertura da auditoria e os avisos pendentes estão descritos nas [notas de dependências](../security/dependency-notes.md).

Os testes de regressão cobrem rollback de sessão após erro de inserção e participação em transação externa; seleção de abas/workspaces; buscador escolhido; correspondência de hosts, entradas URL/Request, reutilização de XHR e beacons; correção da dependência local; e transferências HTTP reais locais, respostas de erro ou interrompidas, política de redirecionamento e colisões de nomes. Os testes de transferência usam HTTP em loopback e diretórios temporários, preservando a pasta Downloads do usuário.

## Verificações nativas

Estas verificações exigem uma área de trabalho interativa e são separadas dos testes e builds:

1. Abra uma página com campo editável, digite um texto, abra outra aba e retorne. O texto e o histórico de voltar/avançar devem permanecer independentes.
2. Hiberne a primeira aba enquanto ela estiver inativa e depois a selecione. Somente essa página deve recarregar. Fechar outra aba em segundo plano não deve despertar páginas hibernadas.
3. Selecione pela Command Bar uma aba de outro workspace. O botão do workspace, o tema e a lista de abas devem concordar. Mover a aba ativa para outro workspace deve preservar essa consistência.
4. Escolha DuckDuckGo e pesquise pelas duas ações da Command Bar: aba atual e nova aba. Ambas devem usar DuckDuckGo.
5. Baixe um recurso HTTP direto. Confira os bytes no disco, nomes sem sobrescrita e erro para uma resposta inválida. O preview no navegador comum não deve fabricar downloads concluídos.
6. Altere a proteção no Windows e confira a preservação dos formulários e do histórico de voltar/avançar. Teste requisições nos documentos atuais e após navegar. Em outros motores, verifique o fallback de recarregamento documentado e a recusa enquanto formulários/mídia/downloads detectados estiverem protegidos.
7. Navegue para um host bloqueado. A última URL confirmada deve permanecer. Uma navegação comum concluída deve adicionar uma entrada ao histórico.

Finalize conferindo `git status`, `git diff`, `git diff --cached` e as regras de ignore para arquivos sensíveis.
