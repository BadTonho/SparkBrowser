# Releases assinadas e GitHub Actions automatizado

As releases são publicadas automaticamente pelo **GitHub Actions** ao enviar tags de versão (por exemplo `v1.0.0` ou `v1.1.0-beta.1`), ou geradas localmente. As releases públicas ficam em `BadTonho/SparkBrowser`. Há suporte tanto para pacotes Windows (instalador NSIS e .zip portátil) quanto para uma ampla variedade de distribuições Linux (.AppImage, .deb, .rpm e .tar.gz portátil). Os dois canais mantêm o identificador `org.spark.browser` e compartilham o perfil/banco existente.

## 1. Releases Automatizadas (GitHub Actions)

Quando você envia uma tag iniciada em `v*`, o workflow `.github/workflows/release.yml` é acionado automaticamente:
1. **Build Windows x64:** Compila o aplicativo, assina o instalador utilizando a chave privada `TAURI_SIGNING_PRIVATE_KEY` salva nos segredos do repositório e gera `SparkBrowser_<version>_x64-setup.exe`, sua assinatura `.sig` e o manifesto `updater.json`. Também empacota `SparkBrowser_<version>_x64-portable.zip` com `spark-browser.exe`, a licença e um marcador `portable.txt`. A cópia portátil não precisa de instalação, compartilha o perfil instalado e não se atualiza sozinha: o instalador a transformaria em uma cópia instalada, então o marcador desliga a atualização automática. Baixe um zip mais novo para atualizá-la. Ela precisa do WebView2 Runtime, que o Windows 10 e 11 já incluem.
2. **Build Linux x64:** Compila o aplicativo com WebKitGTK e gera pacotes para múltiplas distribuições:
   - **`.AppImage` (Universal):** Roda em praticamente todas as distribuições Linux (Fedora, Arch Linux, Manjaro, openSUSE, Pop!_OS, Linux Mint, Debian, Ubuntu, Void, etc.) sem necessitar de root ou instalação.
   - **`.deb` (Debian / Ubuntu):** Pacote nativo para Debian, Ubuntu, Pop!_OS, Linux Mint, Elementary OS e Zorin OS.
   - **`.rpm` (Red Hat / SUSE):** Pacote nativo para Fedora, openSUSE, RHEL, CentOS Stream, AlmaLinux e Rocky Linux.
   - **`portable.tar.gz` (Arquivo Portátil):** Tarball contendo o binário compilado portátil, ideal para empacotadores do Arch Linux (AUR) e ambientes customizados.
3. **Publicação (Modo Rascunho / Draft):** Cria a Release no GitHub como **Draft (Rascunho)**, anexa todos os instaladores compilados, assinaturas e manifestos, e marca automaticamente tags `-beta.*` como **Pre-release**. O modo rascunho permite revisar as notas da versão geradas e inspecionar todos os arquivos anexados privadamente antes de liberar a release para o público.

### Comandos de Instalação no Linux

- **AppImage Universal:**
  ```bash
  chmod +x spark-browser_*.AppImage
  ./spark-browser_*.AppImage
  ```
- **Debian / Ubuntu / Pop!_OS / Mint (`.deb`):**
  ```bash
  sudo apt install ./spark-browser_*_amd64.deb
  ```
- **Fedora / RHEL (`.rpm`):**
  ```bash
  sudo dnf install ./spark-browser-*.rpm
  ```
- **openSUSE (`.rpm`):**
  ```bash
  sudo zypper install ./spark-browser-*.rpm
  ```

### Antes de criar a tag

O workflow de release compila e publica; ele não executa a suíte de regressões nativas (essa suíte é um workflow separado e manual, o `native.yml`, e ainda não passou nos runners do GitHub). Valide primeiro na sua máquina:

O [hook local pre-push](local-hooks.md) exige validação completa, nova compilação nativa isolada com perfil de release e a suíte nativa crítica ao enviar uma tag de versão. Instale-o com `npm run hooks:install` nos checkouts existentes; a instalação do npm também o instala fora da CI. Ele exige checkout limpo, versões iguais e tags apontando para `HEAD`. Os dois jobs de release do GitHub executam verificações de arquitetura/segurança antes de empacotar, mas não conseguem comprovar a execução nativa local. Hooks podem ser ignorados; sempre revise as evidências nativas antes de publicar o rascunho.

1. Execute `npm run validate` e os cenários nativos críticos com `npm run test:native:critical` (veja [testes nativos](native-tests.md)).
2. Confirme que o repositório está limpo e segue a [política de zero vazamentos](rules-and-standards.md#regra-1-zero-arquivos-sensíveis-no-github).
3. A partir da versão 0.1.1, garanta que `changelog/<versão>/` existe com `en.md` e `pt-BR.md` (veja o [guia de changelog](changelog.md)). A versão 0.1.0 não tem changelog.

### Passo a passo para lançar uma atualização:

```bash
# 1. Atualizar a versão dos arquivos (package.json, Cargo.toml e tauri.conf.json)
npm run release:version -- v0.2.0

# 2. Revisar e commitar as alterações
git add .
git commit -m "chore: release v0.2.0"

# 3. Criar a tag e enviar ao GitHub
git tag v0.2.0
git push origin main --tags
```

O GitHub Actions compila e assina os pacotes e cria a release como **rascunho**. Nada fica público até você publicar:

1. Abra o rascunho em **Releases** e confira se ele tem o instalador do Windows (`.exe`, `.sig`, `updater.json`) e todos os pacotes Linux (`.AppImage`, `.deb`, `.rpm`, `.tar.gz`).
2. Revise as notas da versão geradas e edite se precisar.
3. Baixe e teste pelo menos o instalador do Windows e um pacote Linux.
4. Clique em **Publish release**. Tags beta (`-beta.N`) já saem marcadas como **Pre-release**.

## 2. Identidade de assinatura

As assinaturas do atualizador utilizam chaves Minisign Ed25519 pelo plugin oficial do Tauri.
- A **Chave Pública** fica incorporada em `src-tauri/tauri.conf.json` no campo `plugins.updater.pubkey`.
- A **Chave Privada** deve ser guardada estritamente nos Secrets do GitHub como `TAURI_SIGNING_PRIVATE_KEY` (e `TAURI_SIGNING_PRIVATE_KEY_PASSWORD` somente se a chave tiver senha; chaves criadas com `npm run release:setup-signing` não têm). Nunca inclua a chave privada em commits no git!

## 3. Builds manuais locais (Alternativa)

1. Execute `npm run release:version -- v1.1.0-beta.1` ou `npm run release:version -- v1.1.0`. Isso sincroniza metadados npm, configuração Tauri, versão do pacote Cargo e lockfile; não cria uma tag.
2. Execute a [validação](validation.md), revise status/diffs/regras de ignore procurando conteúdo sensível e faça o commit. Crie a tag local correspondente nesse commit, por exemplo `git tag v1.1.0-beta.1`.
3. Execute `npm run release:build -- v1.1.0-beta.1`. Notas opcionais: `npm run release:build -- v1.1.0-beta.1 --notes caminho/notas.md`. O comando exige checkout limpo, versões correspondentes e tag apontando para HEAD. Ele executa `validate --native` e verifica novamente o checkout antes de acessar material de assinatura ou gerar o pacote; falhas interrompem a release. Configure os drivers nativos e mantenha as dependências Rust em cache antes. Ferramentas NSIS do Tauri ausentes são baixadas no primeiro build.
   A chave privada de assinatura é lida de `~/.spark-signing/updater.key`, ou da antiga `~/.veloce-signing/updater.key` quando é lá que ela ainda está; `SPARK_SIGNING_KEY_PATH` substitui as duas (veja [Atualizando a partir do nome Veloce](../features/rename-migration.md)).
4. A saída fica em `.cache/release-artifacts/<tag>/`. Envie somente `SparkBrowser_<version>_x64-setup.exe`, seu `.sig` e `updater.json` à release dessa tag exata. Não envie `build-config.json`, caches, perfis ou arquivos privados de assinatura. O manifesto também incorpora a assinatura.
5. Envie o commit/tag revisados e publique a release no GitHub manualmente. Marque `*-beta.N` como **Pre-release**; estáveis não devem ter essa marcação. Envie os três arquivos antes de publicar. Uma tag isolada não é uma atualização.

Os artefatos têm nomes fixos sem espaços; os manifestos apontam para aquela tag, sem usar `/latest`. Manifestos/instaladores ausentes geram erro de distribuição; manifestos que indicam outra arquitetura são ignorados. A consulta percorre até 2.000 releases e informa erro se ultrapassar esse limite. Os horários de nova tentativa do limite da API GitHub são respeitados. A primeira versão com atualizador exige instalação manual; uma release assinada, completa e superior é necessária para as seguintes. As ferramentas nunca enviam nem publicam arquivos.

## Validação isolada

`cargo test --manifest-path src-tauri/Cargo.toml` inclui um teste de integração que assina bytes artificiais com chaves temporárias e exercita o plugin oficial com assinaturas válidas, adulteração, reaproveitamento com outra versão e downloads interrompidos. Nenhum instalador é executado.

```powershell
npm run test:updater:build
cargo test --manifest-path src-tauri/Cargo.toml --test updater_signatures isolated_nsis_fixtures -- --ignored
npm run test:native:build
```

A ferramenta gera pacotes NSIS beta/estável em `.cache/update-fixtures/`, identificador `org.spark.browser.updater-tests`, com chave temporária apagada ao final. A verificação Rust adicional valida os dois instaladores reais pelo plugin oficial. Isso reutiliza o cache nativo isolado: recompile o executável nativo normal antes da suíte. Nenhum instalador é executado e nenhuma chave de produção é usada. Aplicativos de teste isolados desabilitam consultas reais de atualização ao GitHub.

A execução dos instaladores, atualização/reinício e restauração do perfil instalado ainda exigem uma VM Windows descartável, indisponível na máquina da implementação. Gerar pacotes e verificar assinaturas não equivale a aprovar a execução dos instaladores. Na VM, instale a beta de teste, crie workspaces/notas/abas artificiais, feche, instale a estável superior e confira versão/dados. Não leve perfis nem chaves de produção à VM. A distribuição real pelo GitHub também exige releases assinadas publicadas; a implementação não as publica.
