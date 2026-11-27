# Sincronizar com armazenamento compatível com S3 (futuro)

[English](../../en/roadmap/s3-sync.md)

**Status:** planejado, não iniciado. Vem depois do [Nextcloud](../features/nextcloud-sync.md).

Um modo de sincronização para armazenamento de objetos que fala a API do S3: Cloudflare R2, Backblaze B2, Wasabi, MinIO (servidor próprio) e Amazon S3. Uma única implementação atende todos, e nenhum app precisa ser registrado em lugar nenhum: a pessoa cria um bucket e uma chave de acesso no próprio provedor.

## Por quê

- **Barato ou grátis:** o arquivo de sincronização é pequeno, e o R2 e o B2 têm planos grátis muito acima dele.
- **Servidor próprio:** o MinIO e servidores parecidos rodam em casa ou num VPS.
- **Sem login de conta no Spark:** só uma chave limitada a um bucket, que a pessoa pode revogar.

É pensado para usuários técnicos: criar um bucket e uma chave no painel de um provedor é a parte difícil.

## O que a pessoa preenche

- **Provedor:** atalhos para R2, B2, Wasabi e AWS preenchem o formato do endpoint; **Outro** aceita qualquer endpoint.
- **Endpoint** (`https://`; `http://` só para `localhost`, como no WebDAV) e **região** (`auto` no R2).
- **Bucket** e uma **pasta** opcional (prefixo das chaves) dentro dele.
- **ID da chave de acesso** e **chave de acesso secreta**.
- **Endereços no estilo caminho**, ligado para o MinIO e a maioria dos servidores próprios.

## Plano

1. **Novo destino** `SyncTargetConfig::S3 { endpoint_url, region, bucket, prefix, access_key_id, path_style, auto_save, apply_preferences_on_direct_pull }`. A chave secreta vai para o cofre de credenciais do sistema como uma senha WebDAV lembrada (`backup/credentials.rs`), nunca para o SQLite nem para a interface depois de salva.
2. **Assinatura:** AWS Signature Version 4 escrita em Rust (`backup/s3/sign.rs`, umas 200 linhas com as bibliotecas de criptografia que o Spark já usa), em vez do SDK da AWS, que é grande. O conteúdo é assinado com o SHA-256 dele, não com `UNSIGNED-PAYLOAD`.
3. **Operações:** `PUT`, `GET`, `HEAD`, `DELETE`, `ListObjectsV2` e `CopyObject`, pelo mesmo cliente bloqueante do WebDAV (tempo limite de 20 segundos, sem redirecionamentos).
4. **Versões:** as 5 cópias mais novas em `<prefixo>/versions/browser.spark-backup.<segundos unix>.json`, como numa pasta ou no WebDAV (`sync_versions.rs`). O versionamento do bucket não é usado: nem todo provedor tem, e ele faria a lista depender do provedor.
5. **Testar conexão:** um botão que grava, lê e apaga um pequeno objeto de teste, para que uma chave, região ou permissão errada apareça antes da primeira sincronização.
6. **Interface:** uma seção em `BackupSettings.svelte` como a do WebDAV, com os atalhos, e os mesmos **Automático**, **O que sincroniza**, **Baixar ao abrir o Spark** e **Versões anteriores** dos outros modos.

## Pontos a resolver

- **Relógio fora de hora:** a SigV4 recusa pedidos com mais de 15 minutos de diferença. Mostrar uma mensagem clara para `RequestTimeTooSkewed` em vez de um erro genérico de autenticação.
- **Erros:** dar mensagens próprias para `AccessDenied`, `NoSuchBucket`, `SignatureDoesNotMatch` e `InvalidAccessKeyId` (`backup.s3_*`, nos dois idiomas).
- **Permissões mínimas:** documentar a menor política que funciona (ler, gravar, listar e apagar só no prefixo) na AWS, e os tokens restritos do R2 e do B2.
- **B2:** usar o endpoint compatível com S3, não a API nativa.

## Testes a adicionar

- SigV4 contra os vetores de teste publicados pela AWS.
- Montagem de endereços no estilo virtual-hosted e de caminho, com prefixos que precisam de codificação.
- Uma ida e volta completa (envio, versões, recebimento) contra um servidor de teste local, como o teste de WebDAV em `backup/tests.rs`.
