# Restauração seletiva e modo substituir

Toda restauração (arquivo escolhido, item do histórico ou **Buscar e revisar prévia**) abre o painel de revisão antes de gravar qualquer coisa. O painel mostra quando o backup foi criado e a versão do formato do arquivo, e permite decidir o que será restaurado e como.

## O que restaurar

| Opção | Restaura |
| --- | --- |
| Workspaces, abas e divisões | Workspaces, abas abertas e divisões (dependem umas das outras) |
| Favoritos e pastas | Favoritos e suas pastas |
| Sessões salvas | Sessões nomeadas |
| Notas de página | Notas de texto simples |
| Buscas por site | Buscas por palavra-chave |
| Configurações e aparência | Todas as preferências, exceto atalhos de teclado |
| Atalhos de teclado | O mapa de atalhos |

Todas as opções de dados começam marcadas. As duas opções de preferências começam desmarcadas, então uma restauração nunca altera as configurações locais sem que você peça. Mudar qualquer opção recalcula a prévia, e as contagens sempre descrevem exatamente o que será gravado. É preciso selecionar pelo menos uma opção.

Os grupos de preferências combinam origens: escolher só os atalhos mantém todas as configurações atuais, e escolher só as configurações mantém os atalhos atuais. Uma tarefa de preferências pendente de uma restauração anterior precisa terminar antes.

## Como restaurar

- **Mesclar com os dados atuais** (padrão) mantém os valores atuais nos conflitos e adiciona apenas o que falta, como descrito em [Backup local e importação com revisão](backup.md).
- **Substituir os dados atuais** apaga os itens atuais selecionados e grava o conteúdo do backup no lugar. A tabela ganha a coluna **A remover** e um aviso mostra quantos itens atuais serão apagados permanentemente. O botão de confirmação passa a se chamar **Substituir e importar**.

Substituir vale para favoritos, sessões salvas, notas e buscas por site. Workspaces, abas e divisões são sempre mesclados, porque substituí-los fecharia abas e webviews em uso.

A substituição roda na mesma transação SQLite do restante da importação: se qualquer gravação falhar, nada é apagado. A prévia continua vinculada aos dados atuais, então uma prévia desatualizada precisa ser atualizada antes de confirmar.

## Snapshot de segurança e desfazer

Antes de uma substituição que apagaria algo, o aplicativo salva os dados atuais no histórico de backups locais com o motivo **Antes de substituir** (criptografado quando a criptografia está ativa). Se essa cópia não puder ser gravada, a substituição não acontece. Depois da importação, o resultado mostra **Desfazer substituição…**, que abre a cópia de segurança como uma revisão em modo substituir dos mesmos grupos; confirmar restaura os dados anteriores (e cria uma nova cópia de segurança, então o próprio desfazer pode ser desfeito).

As cópias de segurança contam no limite do histórico e podem ser removidas como qualquer outra entrada, e não reiniciam o agendamento de backup automático. Importações em modo mesclar não apagam nada e não geram cópia.

## Ainda não incluído

- O histórico de navegação não faz parte do formato de backup, então não pode ser restaurado seletivamente.
- O arquivo de backup não registra a versão do aplicativo, apenas a versão do formato do arquivo.
- A seleção vale ao restaurar. Exportações, snapshots do histórico e arquivos de sincronização sempre contêm tudo.

## Implementação

- `RestoreOptions` (`groups`, `mode`) é enviado com `preview_backup_import`; a confirmação reutiliza as opções guardadas com o token da prévia.
- `merge::plan_with` filtra os dados recebidos por grupo, esvazia os grupos substituídos em uma cópia de trabalho e mescla nela, informando `removed` por tipo.
- `commit_backup_import` chama `local::create(..., Reason::PreRestore)` antes quando `BackupService::deletes_local_data` é verdadeiro.
- `backup_repo::commit_with` apaga os grupos substituídos dentro da transação da importação.
- A interface fica em `BackupPreviewPanel.svelte`; o estado está em `backupStore.ts` (`restoreOptions`, `updateRestoreOptions`).
- Testes Rust: `src-tauri/src/backup/selective_tests.rs`.
