# Releases Windows assinadas localmente

As releases são geradas localmente, sem GitHub Actions, servidor próprio de atualização ou push automático. As releases públicas ficam em `BadTonho/navegador`. O formato inicial é NSIS para Windows x64, instalado para o usuário atual. Os dois canais mantêm o identificador `org.veloce.browser` e compartilham o perfil/banco existente.

## Identidade de assinatura

Execute `npm run release:setup-signing` uma vez para uma nova identidade de assinatura. O comando cria `%USERPROFILE%\.veloce-signing\updater.key` e seu `.pub`; depois incorpora a chave pública de verificação. Este checkout já possui chave pública: em outra máquina, restaure o backup da chave privada correspondente; a preparação recusa gerar uma substituta. Faça backup seguro desse diretório: perder a chave privada impede assinar atualizações aceitas pelas instalações existentes. Nunca inclua a chave privada/senha em commits, uploads, mensagens no chat ou logs. A chave pública de verificação pode ser distribuída.

Defina `VELOCE_SIGNING_KEY_PATH` para uma chave existente fora do checkout, mantendo seu `.pub` ao lado dela. A preparação recusa substituir uma chave pública diferente já configurada; a rotação exige migração dos clientes instalados. A chave padrão gerada não possui senha e depende da proteção de acesso do sistema de arquivos. Para uma chave com senha, forneça `TAURI_SIGNING_PRIVATE_KEY_PASSWORD` de forma segura no ambiente do build, nunca em arquivos versionados nem em argumentos de comandos. A ferramenta informa o caminho da chave sem imprimir seu conteúdo.

As assinaturas do atualizador são diferentes dos certificados Windows Authenticode. Esta configuração não fornece certificado de editor Windows nem remove avisos de reputação. A opção `requireSignedVersion` está habilitada: as assinaturas precisam vincular a versão do aplicativo.

## Publicação

1. Execute `npm run release:version -- v1.1.0-beta.1` ou `npm run release:version -- v1.1.0`. Isso sincroniza metadados npm, configuração Tauri, versão do pacote Cargo e lockfile; não cria uma tag.
2. Execute a [validação](validation.md), revise status/diffs/regras de ignore procurando conteúdo sensível e faça o commit. Crie a tag local correspondente nesse commit, por exemplo `git tag v1.1.0-beta.1`.
3. Execute `npm run release:build -- v1.1.0-beta.1`. Notas opcionais: `npm run release:build -- v1.1.0-beta.1 --notes caminho/notas.md`. O comando exige checkout limpo, versões correspondentes e tag apontando para HEAD. Ferramentas NSIS do Tauri ausentes são baixadas no primeiro build.
4. A saída fica em `.cache/release-artifacts/<tag>/`. Envie somente `VeloceBrowser_<version>_x64-setup.exe`, seu `.sig` e `updater.json` à release dessa tag exata. Não envie `build-config.json`, caches, perfis ou arquivos privados de assinatura. O manifesto também incorpora a assinatura.
5. Envie o commit/tag revisados e publique a release no GitHub manualmente. Marque `*-beta.N` como **Pre-release**; estáveis não devem ter essa marcação. Envie os três arquivos antes de publicar. Uma tag isolada não é uma atualização.

Os artefatos têm nomes fixos sem espaços; os manifestos apontam para aquela tag, sem usar `/latest`. Manifestos/instaladores ausentes geram erro de distribuição; manifestos que indicam outra arquitetura são ignorados. A consulta percorre até 2.000 releases e informa erro se ultrapassar esse limite. Os horários de nova tentativa do limite da API GitHub são respeitados. A primeira versão com atualizador exige instalação manual; uma release assinada, completa e superior é necessária para as seguintes. As ferramentas nunca enviam nem publicam arquivos.

## Validação isolada

`cargo test --manifest-path src-tauri/Cargo.toml` inclui um teste de integração que assina bytes artificiais com chaves temporárias e exercita o plugin oficial com assinaturas válidas, adulteração, reaproveitamento com outra versão e downloads interrompidos. Nenhum instalador é executado.

```powershell
npm run test:updater:build
cargo test --manifest-path src-tauri/Cargo.toml --test updater_signatures isolated_nsis_fixtures -- --ignored
npm run test:native:build
```

A ferramenta gera pacotes NSIS beta/estável em `.cache/update-fixtures/`, identificador `org.veloce.browser.updater-tests`, com chave temporária apagada ao final. A verificação Rust adicional valida os dois instaladores reais pelo plugin oficial. Isso reutiliza o cache nativo isolado: recompile o executável nativo normal antes da suíte. Nenhum instalador é executado e nenhuma chave de produção é usada. Aplicativos de teste isolados desabilitam consultas reais de atualização ao GitHub.

A execução dos instaladores, atualização/reinício e restauração do perfil instalado ainda exigem uma VM Windows descartável, indisponível na máquina da implementação. Gerar pacotes e verificar assinaturas não equivale a aprovar a execução dos instaladores. Na VM, instale a beta de teste, crie workspaces/notas/abas artificiais, feche, instale a estável superior e confira versão/dados. Não leve perfis nem chaves de produção à VM. A distribuição real pelo GitHub também exige releases assinadas publicadas; a implementação não as publica.
