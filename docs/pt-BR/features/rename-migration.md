# Atualizando a partir do nome Veloce

O aplicativo se chamava Veloce antes de virar Spark. Tudo o que foi salvo com o nome antigo continua funcionando: na primeira execução de uma versão Spark, os dados são movidos e convertidos uma única vez, sem perguntar.

## O que é migrado sozinho

| Dado | O que acontece |
| --- | --- |
| Pastas do perfil (`org.veloce.browser` → `org.spark.browser`) | Renomeadas antes de qualquer webview abrir, então favoritos, configurações e o perfil do WebView (cookies, armazenamento dos sites) vão junto. No Windows são movidos o Roaming e o Local; no Linux, dados, configuração e cache. Nada é movido se a pasta nova já existir. |
| Arquivo do banco (`veloce_browser.db` → `spark_browser.db`) | Renomeado junto com os arquivos `-wal` e `-shm`. |
| Configurações da interface salvas (chaves `veloce_*` → `spark_*`) | Copiadas na inicialização, antes de qualquer configuração ser lida; a chave antiga é removida. Um valor que já existe no nome novo nunca é sobrescrito. |
| Abas salvas, sessões e abas fechadas recentemente com `veloce://newtab` | Reescritas para `spark://newtab` uma única vez. O endereço antigo continua sendo reconhecido se aparecer num arquivo importado. |
| Senhas do WebDAV e da criptografia de backup no cofre de credenciais do sistema | Copiadas dos nomes antigos para os novos, e as entradas antigas são removidas. Se o cofre estiver bloqueado, a próxima inicialização tenta de novo. |
| Gatilhos do índice de busca em bancos existentes | Eles chamam `veloce_search_fold`, então o app continua registrando essa função ao lado de `spark_search_fold`. |

## Backups e arquivos de sincronização

- Os backups continuam abrindo: arquivos `.veloce-backup.json`, documentos com a marca `veloce-backup` e arquivos criptografados com a marca `veloce-backup-encrypted` são todos lidos. Os arquivos novos usam `.spark-backup.json`, `spark-backup` e `spark-backup-encrypted`.
- A criptografia em si não muda. Os dados associados que entram na autenticação de todo arquivo criptografado mantêm o texto antigo de propósito, porque trocá-lo tornaria impossível abrir os backups criptografados existentes.
- O histórico local lista entradas `…veloce-backup.json` e `…spark-backup.json`; os novos snapshots usam o nome novo.
- Os destinos de sincronização agora gravam `browser.spark-backup.json`. Ao buscar, procuram primeiro o nome novo e depois `browser.veloce-backup.json`: em pasta sincronizada, no WebDAV (quando o endereço é uma pasta) e no Google Drive. No Drive a busca aceita os dois nomes, então um arquivo existente é encontrado e atualizado no mesmo lugar, sem aparecer uma segunda cópia. O nome antigo da pasta do Drive, `Veloce Browser Backup`, também continua sendo procurado.

## O que não é levado

- **Armazenamento da página de nova aba.** A página de nova aba é servida de um endereço próprio (`spark-internal://…`), que é uma origem diferente da anterior. Papel de parede e configurações do relógio são guardados por origem e voltam ao padrão.
- **Downloads incompletos.** Os arquivos parciais de downloads retomáveis passaram de `.veloce-partials` para `.spark-partials` na pasta de downloads. Um download que ainda estava em andamento não consegue retomar; downloads concluídos não são afetados, e a pasta oculta antiga pode ser apagada.
- Qualquer coisa criada por uma versão que nunca rodou, como perfis de teste: as versões de teste nativo usam o identificador próprio `org.spark.browser.native-tests`.

## Para quem desenvolve

- A chave de assinatura das versões pode continuar em `~/.veloce-signing`: `release:build` e `release:setup-signing` usam `~/.spark-signing` quando ela existe e, caso contrário, a pasta antiga, e respeitam `SPARK_SIGNING_KEY_PATH` ou o antigo `VELOCE_SIGNING_KEY_PATH`. Mover a chave é opcional.
- As variáveis de ambiente dos testes agora são `SPARK_NATIVE_*`; pacote `spark-browser`, biblioteca `spark_browser_lib`.
- Código: `src-tauri/src/legacy.rs` (perfil e banco), `backup/credentials.rs` (segredos), `database/connection.rs` (reescrita do endereço), `src/lib/services/storage.js` (chaves de configuração). Testes: `legacy_tests.rs`, `tests/rename-compat.test.mjs` e o módulo `rename_tests` em `connection.rs`.
- Os nomes anteriores à troca podem ser removidos depois de algumas versões, quando não se esperar mais um perfil anterior à troca; cada parte acima está isolada para isso.
