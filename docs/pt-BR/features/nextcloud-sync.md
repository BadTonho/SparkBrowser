# Sincronizar com o Nextcloud

[English](../../en/features/nextcloud-sync.md)

**Configurações → Dados e backup → Sincronização → Nextcloud** sincroniza com o seu próprio servidor Nextcloud (ou o de um provedor). Você só digita o endereço do servidor: não há endereço WebDAV para montar, senha para digitar no Spark nem app para registrar.

## Entrar

1. Escolha **Nextcloud** como modo de sincronização e digite o endereço do servidor, como `nuvem.exemplo.com`. O `https://` é adicionado quando falta, e um endereço colado de uma página do Nextcloud (`.../index.php/apps/files/...`, `.../login`) é cortado até o servidor. `http://` simples só é aceito para `localhost`, como no WebDAV.
2. Clique em **Entrar no Nextcloud**. As configurações fecham e a página de login do servidor abre em uma guia.
3. Entre lá e permita o Spark. O Spark percebe em cerca de 2 segundos, fecha essa guia e abre as configurações de novo em Dados e backup, já conectado.

Enquanto o Spark espera, o painel de sincronização avisa e oferece **Cancelar login**. Fechar a guia de login também cancela. Um login deixado pela metade expira depois de 20 minutos, como o do próprio Nextcloud.

Depois de conectado, o painel mostra a conta e o servidor em uso. **Entrar de novo** pega uma nova senha de app (por exemplo depois de remover o Spark no Nextcloud), e as opções de **Automático** valem na hora.

## Onde o arquivo fica

O Spark cria uma pasta **Spark** nos seus arquivos (se ela ainda não existir) e guarda `browser.spark-backup.json` nela. A partir daí a sincronização é a mesma do [WebDAV](backup.md): os mesmos envios automáticos, modos de recebimento, **O que sincroniza**, **Baixar ao abrir o Spark** e **Versões anteriores** (as 5 cópias mais novas em `Spark/versions`). O arquivo é criptografado do mesmo jeito quando a criptografia de backup está ligada.

## Segurança

- O Spark usa o login para apps do Nextcloud (Login Flow v2). A sua senha do Nextcloud só é digitada na página do seu servidor; o Spark recebe uma **senha de app** feita só para ele.
- A senha de app fica no cofre de credenciais do sistema (Gerenciador de Credenciais do Windows ou Secret Service no Linux), como uma senha WebDAV lembrada, e nunca chega à interface. O token de consulta do login também fica no Rust.
- O Spark só segue uma página de login e um endereço de consulta no mesmo servidor (mesmo esquema, host e porta) que você digitou, e nunca segue redirecionamentos.
- O Nextcloud lista o Spark como **Spark Browser (sync)** em **Configurações → Segurança → Dispositivos e sessões**, onde você pode removê-lo quando quiser. O Spark passa a receber um erro de autenticação até você entrar de novo.
- Trocar para outro modo de sincronização esquece a senha de app neste computador. Isso não a revoga no servidor: remova-a lá se não for mais usar.

## Como funciona

- Rust: `src-tauri/src/backup/nextcloud.rs` limpa o endereço do servidor, faz o login (`POST /index.php/login/v2`, depois consulta o endereço devolvido com o token até ele responder com o nome de login e a senha de app) e cria a pasta (`MKCOL`). Os comandos ficam em `src-tauri/src/commands/backup_nextcloud.rs`: `nextcloud_login_start`, `nextcloud_login_poll` e `nextcloud_login_cancel`.
- Quando a pessoa permite o Spark, a consulta salva um destino WebDAV (`direct_webdav`) apontado para `<servidor>/remote.php/dav/files/<nome de login>/Spark/`, com a senha de app lembrada e `nextcloud_server` preenchido. É esse campo que faz a interface mostrar o destino como Nextcloud. O `save_config` confere que ele é um servidor válido no mesmo host do endereço WebDAV.
- Interface: `src/lib/stores/nextcloudLoginStore.ts` abre a guia de login, consulta a cada 2 segundos e desiste quando a guia é fechada. O painel de sincronização fica em `BackupSettings.svelte`.
- Falhas são registradas no log de erros como Warning, com a etapa que falhou (início ou login), e o painel mostra a mensagem.

Testes: `src-tauri/src/backup/nextcloud.rs` (limpeza do endereço, endereço da pasta, conferência da origem) e `tests/backup-nextcloud.test.mjs` (comandos registrados de ponta a ponta, mensagens nos dois idiomas, senha de app fora da interface).
