# Integridade e versões do backup

## Checksum de integridade

Os backups gravados por esta versão usam o formato versão 6. As versões 5 e 6 guardam um `checksum` SHA-256 no nível superior do arquivo. Ele cobre o documento inteiro (formato, versão, data de criação, dados e preferências) em forma canônica: chaves ordenadas e espaços ignorados, então reformatar o arquivo em outra ferramenta não o invalida.

Ao abrir um arquivo versão 5 ou 6, seja de um arquivo, do histórico ou de um destino de sincronização, o checksum é verificado antes de qualquer outra leitura:

- **Divergência** (`backup.checksum_mismatch`): o arquivo está danificado ou foi alterado depois de salvo. Nada é importado.
- **Checksum ausente** em um arquivo versão 5 ou 6 (`backup.checksum_missing`): tratado do mesmo modo, porque removê-lo pularia a verificação.
- Backups criptografados são verificados após a descriptografia, além da autenticação que o AES-GCM já oferece.

Arquivos das versões 1 a 4 não têm checksum e continuam aceitos sem ele. O checksum detecta corrupção acidental e edições manuais; não é uma assinatura, então quem consegue reescrever o arquivo também consegue recalculá-lo. Use criptografia quando adulteração for uma preocupação.

## Versões e atualizações

A `version` do arquivo é a versão do esquema. As versões suportadas vão de 1 a 6.

- Um arquivo com **versão maior** do que o aplicativo entende é recusado com `backup.newer_version` ("criado por uma versão mais nova, atualize o aplicativo"), antes da leitura estrita, em vez de um erro genérico de arquivo inválido.
- Arquivos antigos são validados com as regras da própria versão e depois atualizados para o formato de dados atual por `backup/migrate.rs`, um passo por versão atravessada. O documento mantém a `version` original, que o painel de revisão exibe.

| Passo | O que cobre |
| --- | --- |
| 1 → 2 | Preferências opcionais de zoom por site; ausente significa nenhuma |
| 2 → 3 | `pip_window_shadow`; ausente significa `false` |
| 3 → 4 | Divisões ganham grupos explícitos de abas esquerda/direita; divisões antigas colocam as demais abas à esquerda |
| 4 → 5 | Apenas o checksum do arquivo |
| 5 → 6 | Preferência de exibição das configurações; ausente significa pop-up |

Passos que só acrescentam campos opcionais dependem dos valores padrão da desserialização e não precisam de código. Quando o formato mudar de novo, aumente `CURRENT_VERSION`, aceite a nova versão na validação e adicione um passo em `migrate::upgrade`.

## Implementação

- `files::encode` calcula o checksum; `files::decode_with` o verifica por meio de `verify_integrity`.
- `BackupService::select` e as prévias estruturadas chamam `migrate::upgrade`.
- Testes: `src-tauri/src/backup/integrity_tests.rs`.
