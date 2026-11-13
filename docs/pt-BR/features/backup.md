# Backup local e importação com revisão

A preferência booleana opcional `auto_skip_ads_enabled` também é incluída. Backups antigos usam false; aplicar as preferências revisadas atualiza o armazenamento local e a política em execução. Importar somente dados mantém a escolha local de auto skip. Veja [Auto skip ads](auto-skip-ads.md).

O contrato de atalhos inclui a ação de Picture-in-Picture e é conferido contra a lista de ações padrão da interface.

Configurações → Dados e backup exporta um arquivo `.veloce-backup.json` versionado pelo seletor de arquivos do sistema. O recurso está disponível no aplicativo desktop; o preview no navegador o mostra como indisponível. O formato é `veloce-backup`, versão 4, limitado a 32 MiB. As versões 1–3 continuam importáveis. A versão 4 acrescenta os grupos de abas esquerdo/direito da divisão e remapeia cada integrante ao importar um novo workspace; divisões antigas colocam as demais abas à esquerda. A versão 2 introduziu preferências opcionais de zoom por site; origens locais existentes prevalecem nos conflitos. A versão 3 acrescenta a preferência booleana `pip_window_shadow`, com padrão `false` quando ausente. Ela só é aplicada quando **Aplicar preferências do backup** está marcado; importar somente os dados mantém a escolha local de sombra. Nenhum serviço em nuvem é usado.

O backup contém favoritos/pastas, workspaces/cores, sessões nomeadas, notas de texto simples, buscas por palavra-chave, ordem/fixação/seleção de abas abertas, divisões e preferências persistentes de interface/navegação/hibernação/idioma/atalhos/canal de atualização. Cookies, armazenamento de senhas, permissões nativas, histórico, abas fechadas, registros/arquivos de downloads e chaves de assinatura não são incluídos. Formulários, rolagem e histórico interno das páginas não fazem parte do snapshot. Somente endereços HTTP/HTTPS sem credenciais incorporadas são aceitos.

A exportação salva notas pendentes e gravações de navegação, captura um snapshot consistente do banco/sessão e grava um arquivo temporário no diretório de destino. O arquivo completo é sincronizado e substitui atomicamente o destino escolhido. Falhas de validação ou gravação mantêm o backup anterior. Use um nome terminado em `.veloce-backup.json`; o caminho nunca muda silenciosamente após a confirmação de sobrescrita.

A importação primeiro mostra contagens, adições, conflitos ignorados e valores das preferências. **Aplicar preferências do backup** começa desmarcado. Nomes de workspaces/pastas são comparados sem diferenciar maiúsculas; endereços de favoritos/notas e palavras-chave identificam dados existentes. Os valores atuais prevalecem. Sessões de conteúdo idêntico são ignoradas; sessões diferentes de mesmo nome recebem sufixo único. Notas ignoram fragmentos do endereço, mantendo os parâmetros de consulta.

A seleção de arquivo mantém um documento validado no backend nativo e retorna um token opaco de seleção. A interface recebe resumos da prévia e preferências. Substituir ou cancelar uma seleção, ou concluir sua importação, libera o documento retido. A validação de prévia desatualizada usa um digest SHA-256 calculado por streaming dos dados/preferências atuais, sem reter outra cópia serializada. Importações programáticas estruturadas continuam disponíveis.

As abas atuais mantêm IDs, ordem e seleção. O importador compara as contagens de ocorrências de cada endereço em cada workspace e adiciona somente as que faltam, com IDs novos, preservando duplicatas intencionais. As novas abas ficam hibernadas. Pares divididos são remapeados somente em workspaces novos; divisões existentes são mantidas. Reimportar o mesmo backup não acrescenta duplicatas.

A prévia fica vinculada aos dados e preferências atuais. Se houver alterações antes da confirmação, atualize a prévia. Os dados e a tarefa opcional de preferências são confirmados em uma transação SQLite. Preferências são aplicadas depois; falhas deixam uma tarefa persistente e botão de nova tentativa, inclusive após reiniciar. A nova tentativa aplica preferências sem importar dados novamente. Conclua uma tarefa pendente antes de importar outro conjunto de preferências. Arquivos de backup e temporários são ignorados pelo Git.

Testes Rust cobrem ida e volta do arquivo, versões/referências/tamanho inválidos, substituição atômica, conflitos, contagens de ocorrências, remapeamento de IDs/divisões, rollback e isolamento da nova tentativa de preferências. `npm run test:native -- --case backup` exercita comandos reais, prévia desatualizada, hibernação, importação repetida e persistência após reinício. A seleção nativa usa o [plugin de diálogos do Tauri](https://v2.tauri.app/plugin/dialog/) pelo Rust, sem conceder seus comandos JavaScript de arquivos/diálogos.

## Sincronização automática isolada (Google Drive na Nuvem, Pasta e Conexão Direta WebDAV)

Configurações → Dados e backup também inclui a **Sincronização automática isolada**, permitindo manter o arquivo `browser.veloce-backup.json` sincronizado no armazenamento do próprio usuário sem servidores do Spark nem contas em consoles de nuvem de desenvolvedor:

1. **Modo Google Drive na nuvem (usa o login do Gmail no navegador):**
   - Usa os cookies de login da conta Google/Gmail do navegador (`drive.google.com`) para criar/localizar automaticamente a pasta dedicada **`Spark Backup`** no `Meu Drive` e enviar/baixar o arquivo `browser.veloce-backup.json` dentro dela, sem precisar de pasta no PC nem de projeto no Google Cloud Console. Ao buscar um backup, caso o arquivo ainda não esteja dentro da pasta `Spark Backup`, o navegador também verifica a raiz do `Meu Drive` para manter compatibilidade com backups anteriores.
   - Exibe na interface o e-mail da conta Google conectada (`connected_account`) e oferece o botão **Escolher / Trocar conta do Google (Gmail)**, que abre o `AccountChooser` do Google (`accounts.google.com/AccountChooser`) e sincroniza automaticamente assim que a aba do Google Drive da conta escolhida é carregada.
   - Com o salvamento automático ativo, um ciclo de 1 minuto verifica o fingerprint SHA-256 em streaming dos dados e preferências atuais e envia para o Google Drive **apenas quando houver alterações**, além de sincronizar ao fechar o navegador e pelo botão **Sincronizar agora (Salvar)**.
2. **Modo Pasta sincronizada (OneDrive, Mega, Dropbox, Pendrive):**
   - O usuário escolhe um diretório local pelo seletor nativo de pastas (`choose_backup_sync_folder`).
   - Com o salvamento automático ativo, alterações em favoritos, workspaces, abas, divisões, notas, atalhos ou preferências gravam atomicamente `browser.veloce-backup.json` na pasta escolhida a cada alteração, além de salvar ao fechar o navegador e pelo botão **Sincronizar agora (Salvar)**.
3. **Modo Conexão direta (WebDAV / Nuvem própria):**
   - O usuário informa uma URL WebDAV `https://` (`http://127.0.0.1` / `localhost` é aceito apenas para testes locais), usuário e senha de aplicativo ou token.
   - O usuário pode escolher entre **lembrar a senha neste computador** (Gerenciador de Credenciais do Windows ou Secret Service do Linux por meio de `secret-tool`) ou mantê-la **apenas na memória durante a sessão atual**.
   - Com o salvamento automático ativo, um ciclo de 1 minuto verifica o fingerprint SHA-256 em streaming dos dados e preferências atuais e envia (`PUT`) para o servidor WebDAV **apenas quando houver alterações**, além de sincronizar ao fechar o navegador e pelo botão **Sincronizar agora (Salvar)**.
4. **Duas opções de recebimento (`pull_backup_sync`):**
   - **Buscar e revisar prévia:** lê/baixa o backup e abre a tabela padrão de revisão antes de confirmar a importação.
   - **Buscar e aplicar direto (Auto-Merge):** valida e mescla o backup imediatamente em um único passo, podendo aplicar também as preferências quando marcado.
5. **Fronteira Zero-Leak:** Caminhos de pasta de sincronização, URLs WebDAV e credenciais são configurações estritamente locais e nunca são serializados dentro do `BackupDocument`.

## Segurança da importação e das contas

A importação preserva todas as abas existentes em memória, inclusive descartáveis, sua ordem, seleção ativa, participação em divisão e identidade de container de conta. As identidades existentes são preservadas na transação SQLite; abas importadas usam a identidade padrão porque backups não incluem dados de containers de conta.

A sincronização com o Drive exige o fluxo explícito **Escolher / Trocar conta Google**. Ela vincula o perfil persistente e o índice de conta selecionados e verifica o e-mail detectado antes da transferência. Abas descartáveis e outros perfis ativos nunca são alternativas. Se o perfil escolhido não tiver uma aba residente no Windows, reabra uma aba nesse perfil. Instalações existentes precisam escolher a conta Google uma vez após a atualização.

Senhas WebDAV lembradas não entram em novos valores de preferências SQLite. No Linux, é necessário `secret-tool` (pacote `libsecret-tools` no Debian/Ubuntu) e um provedor de Secret Service desbloqueado. Falhas são informadas sem recorrer a texto simples. Senhas antigas no SQLite são removidas com exclusão segura ativada e migradas para o cofre do sistema na inicialização; se a migração falhar, permanecem somente na memória do processo durante essa sessão. Cópias históricas do banco e backups não podem ser apagados por essa migração. Alterar o servidor WebDAV ou o usuário não reutiliza a senha do destino anterior.

Veja [Backups automáticos, histórico e criptografia](backup-automation.md) para snapshots agendados, restauração pelo histórico e proteção por senha, e [Restauração seletiva e modo substituir](selective-restore.md) para escolher o que uma restauração altera.
