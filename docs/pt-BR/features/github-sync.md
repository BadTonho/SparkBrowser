# Backup no GitHub (repositório privado)

[English](../../en/features/github-sync.md)

**Configurações → Dados e backup → Sincronização** tem um segundo cartão, **GitHub**, ao lado do cartão de sincronização. Ele guarda um backup extra e criptografado num repositório privado da sua conta do GitHub, **ao mesmo tempo** que a sincronização em uso (Google Drive, pasta, Nextcloud ou WebDAV), com configuração, estado e erros próprios. Ele usa um token que você mesmo cria, então nenhum app precisa ser registrado.

## Configurar

1. Ligue a **criptografia dos backups** (Backups automáticos → Criptografia). O backup no GitHub se recusa a conectar ou enviar sem ela, e o cartão avisa.
2. No GitHub, crie um repositório **privado** com um README (um repositório vazio não recebe arquivos pela API).
3. Crie um token em **Settings → Developer settings → Personal access tokens → Fine-grained tokens** com acesso **só a este repositório** e a permissão **Contents** em **Read and write**.
4. Abra o cartão **GitHub**, digite o repositório como `dono/nome` (ou cole o endereço dele), mantenha a branch `spark-sync` ou digite outra, cole o token e clique em **Conectar**.

**Conectar** confere tudo no GitHub antes de salvar: o token funciona, o repositório é privado, o token pode gravar nele, a branch não é a padrão e a branch só tem arquivos do Spark. O primeiro backup é enviado logo em seguida.

O cartão fechado mostra o logo do GitHub (na cor do texto, para aparecer nos dois temas), **Conta conectada** enquanto houver um token salvo e o horário do último backup. A borda mostra o mesmo anel do cartão de sincronização: verde enquanto envia, amarelo com o envio automático desligado ou a sincronização automática pausada, vermelho depois de um erro.

Para trocar a branch ou o token depois, edite os campos e clique em **Salvar alterações** (deixar o campo do token vazio mantém o token salvo). **Desconectar** esquece o repositório e o token neste computador; o token continua válido no GitHub até você apagá-lo lá.

## Com que frequência envia

Um backup completo vai para o repositório **no máximo uma vez por hora**, quando algo mudou. A hora é contada a partir do último backup salvo no disco (ou da última tentativa nesta execução, para uma falha não ser repetida a cada minuto), então reabrir o Spark não envia de novo. Fechar o Spark não acrescenta um envio. Pausar a sincronização automática (Frequência e pausa) também pausa este envio.

No cartão aberto:

- **Enviar agora** manda um backup na hora, mesmo que nada tenha mudado.
- **Revisar** lê o backup do repositório e abre no painel de revisão de Dados e backup: nada muda até ser confirmado lá.
- **Versões anteriores** lista as cópias guardadas na branch e abre uma no mesmo painel de revisão.

O backup no GitHub sempre leva todos os grupos (workspaces e abas, favoritos, sessões, notas, buscas por site, configurações, atalhos): **O que sincroniza** só vale para o destino de sincronização.

## O que vai para o repositório

O Spark só grava na branch dele (`spark-sync` por padrão) e nunca na branch padrão:

- `browser.spark-backup.json`: o backup criptografado;
- `versions/browser.spark-backup.<segundos unix>.json`: as 5 cópias anteriores mais novas.

Cada envio substitui a branch por **um único commit sem histórico** (uma atualização forçada) com esses arquivos. O arquivo criptografado muda inteiro a cada envio, então guardar cada envio como um commit faria o repositório crescer a cada hora; com um único commit, o GitHub descarta as cópias antigas sozinho.

## Conferências de segurança

- **Antes de cada envio**, o Spark confere de novo se o repositório é privado, se o token pode gravar e se a branch não é a padrão. Um repositório que vire público depois para os backups com uma mensagem clara, e nada é enviado.
- Uma branch com algum arquivo que o Spark não gravou nunca é substituída (`backup.github_branch_in_use`). Uma branch protegida é informada em vez de forçada.
- Sem criptografia nada é enviado, mesmo que a criptografia seja desligada depois de conectar.
- O token fica no cofre de credenciais do sistema (Gerenciador de Credenciais do Windows ou Secret Service no Linux) numa chave própria, separada da senha do destino de sincronização, e nunca é mostrado de novo. Tokens fine-grained vencem; quando um vence, o cartão diz que o token foi recusado e pede um novo.
- Os pedidos vão só para `https://api.github.com`, sem seguir redirecionamentos.

## Como funciona

- Rust: `src-tauri/src/backup/github.rs` é o cliente do GitHub: confere o repositório, a branch e o token, lê a branch (`git/ref`, `git/commits`, `git/trees`) e envia pela Git Data API (um blob para o arquivo, uma árvore que reaproveita os blobs das versões mantidas, um commit sem pais e uma atualização forçada da branch). `plan_tree` decide quais arquivos o commit leva e recusa uma branch com outros arquivos.
- `src-tauri/src/backup/github_backup.rs` é o segundo destino: a configuração, o último envio e o fingerprint das mudanças ficam no banco do perfil em chaves próprias, o token no cofre de credenciais e o último erro na memória. Ele não mexe no destino de sincronização (`sync.rs`).
- Comandos (`src-tauri/src/commands/backup_github.rs`): `get_github_backup_state`, `connect_github_backup` (confere no GitHub antes de salvar), `set_github_backup_auto`, `disconnect_github_backup`, `push_github_backup`, `review_github_backup`, `list_github_backup_versions` e `review_github_backup_version`.
- Interface: `src/lib/stores/githubBackupStore.ts` carrega o estado quando o Spark abre e confere a cada minuto se um backup de hora em hora está na vez (`githubDue`); as revisões passam por `reviewFrom` em `backupStore.ts`, o mesmo painel de revisão do destino de sincronização. O cartão fica em `BackupSettings.svelte`, ao lado do cartão de sincronização; quando há espaço, ele abre para a esquerda para ficar dentro da categoria.
- Falhas são registradas no log de erros com o repositório e a etapa (conectar, enviar ou baixar), e cada uma tem sua própria mensagem nos dois idiomas (`backup.error_github_*`).

Testes: `src-tauri/src/backup/github.rs` (conferência de repositório, branch e token, quais arquivos um envio mantém, recusa de branch com outros arquivos, ordem das versões) e `tests/backup-github.test.mjs` (comandos registrados de ponta a ponta, o cartão ao lado do destino de sincronização, a regra de uma hora, as conferências de segurança, mensagens nos dois idiomas).
