# Backups automáticos, histórico e criptografia

Configurações → Dados e backup inclui um painel de **Backups automáticos** construído sobre a importação com revisão descrita em [Backup local e importação com revisão](backup.md). Somente em versões desktop.

## Backups agendados

- **Fazer backup automaticamente** mantém um histórico rotativo no diretório de dados do aplicativo (`backups/`). A frequência é a cada 6 horas, diária ou semanal.
- **Também fazer backup ao fechar o navegador** adiciona um snapshot durante o encerramento. Uma falha nunca atrasa nem bloqueia o fechamento do navegador.
- **Fazer backup agora** cria um snapshot sob demanda e sempre grava um.
- A interface verifica o agendamento uma vez por minuto e logo após a inicialização; assim, um backup que venceu com o navegador fechado é feito na próxima abertura. Snapshots agendados e ao fechar comparam a impressão SHA-256 em fluxo dos dados e preferências com o snapshot anterior e ignoram conteúdo idêntico, apenas renovando o relógio do agendamento. Após uma falha, a próxima tentativa é adiada em 15 minutos.
- Os snapshots são capturados com as mesmas regras de descarga, bloqueio de sessão e consistência da exportação manual.

## Histórico e restauração

- Cada snapshot se chama `backup-<epoch>-<manual|scheduled|exit>.spark-backup.json`. O painel lista data, origem, tamanho e se está criptografado, do mais novo ao mais antigo.
- **Versões a manter** (1–100, padrão 10) remove os snapshots mais antigos após cada backup. Reduzir o valor remove imediatamente.
- **Apagar backups com mais de (dias)** (1–3650, vazio = nunca) acrescenta um limite de idade além da quantidade. Os dois limites valem juntos, e o backup mais recente é sempre mantido, então a retenção nunca esvazia o histórico. Os limites são aplicados após cada backup (inclusive uma execução agendada que não encontra mudanças), ao salvar o agendamento e na primeira leitura do histórico após iniciar, então o limite de idade funciona mesmo quando nenhum backup está pendente.
- **Cópias de segurança “Antes de substituir”** (veja [Restauração seletiva e modo substituir](selective-restore.md)) não contam no limite de versões nem na idade máxima. Elas têm política própria: as 5 mais recentes, por no máximo 30 dias.
- A retenção cobre apenas o histórico local. Os destinos de sincronização (Google Drive, pasta sincronizada, WebDAV) guardam um único `browser.spark-backup.json` que é sobrescrito a cada sincronização, então não há versões antigas para limpar.
- **Status** fica no topo do painel Backups automáticos: um selo (**Os backups estão funcionando**, **O último backup falhou** ou **Nenhum backup ainda**), o horário do último backup, o próximo horário agendado e a quantidade e o tamanho total dos backups guardados. "Último backup" é a entrada regular mais recente do histórico; as cópias "Antes de substituir" não contam.
- Uma tentativa com falha (manual, agendada, ao fechar ou a cópia de segurança antes de substituir) fica registrada entre reinicializações até que uma tentativa posterior tenha sucesso, com o horário e o motivo traduzido. Só um código de erro estável é guardado, nunca a mensagem nativa, para que caminhos de arquivos não fiquem no banco de configurações. Com o agendamento ligado, o painel avisa que o Spark tentará novamente de forma automática. Uma execução agendada que não encontra mudanças conta como sucesso.
- **Restaurar…** carrega o snapshot no fluxo de revisão existente: contagens, adições, conflitos ignorados, preferências opcionais e proteção contra prévia desatualizada continuam valendo. Nada muda até **Confirmar importação**. Restaurar continua sendo uma mesclagem: valores atuais vencem em conflitos.
- **Excluir** exige um segundo clique no mesmo botão para confirmar.
- Os identificadores dos snapshots são validados contra o padrão de nome gerado, então um nome de arquivo vindo da interface nunca alcança fora do diretório de backups. Arquivos de terceiros e exportações temporárias nesse diretório são ignorados.

## Proteção por senha

- **Proteger backups com senha** criptografa todo backup que o aplicativo grava: exportações manuais, snapshots do histórico e o destino de sincronização (Google Drive, pasta sincronizada e WebDAV). O arquivo mantém a extensão `.spark-backup.json`.
- Formato `spark-backup-encrypted` (envelope versão 1): a chave é derivada com Argon2id (64 MiB, 3 iterações, 1 linha, salt aleatório de 16 bytes) e o documento é selado com AES-256-GCM usando um nonce aleatório de 12 bytes. Os parâmetros de custo ficam no envelope e são limitados na leitura, então um arquivo hostil não consegue exigir memória ou tempo ilimitados. Senha incorreta e arquivo modificado são indistinguíveis por projeto.
- A senha fica somente no cofre de credenciais do sistema (Gerenciador de Credenciais do Windows, ou Secret Service no Linux via `secret-tool`), sob uma chave separada do segredo WebDAV. Ela nunca entra no SQLite, nos backups, nos logs nem na cópia do perfil privado. As senhas têm de 8 a 256 caracteres.
- Se a criptografia estiver ativa, mas a senha não puder ser lida, a gravação falha com um erro explícito em vez de produzir um arquivo sem criptografia.
- **Escolher backup…**, **Restaurar…** e as buscas de sincronização tentam primeiro a senha armazenada. Quando ela não existe ou não confere (por exemplo, um backup antigo criado com outra senha), a interface pede a senha; uma tentativa errada mantém o arquivo selecionado para nova tentativa, e **Cancelar** o libera. Backups sem criptografia continuam legíveis com ou sem senha.
- Alterar a senha afeta apenas novos backups: os antigos mantêm a senha com que foram criados. **Perder a senha torna os backups criptografados irrecuperáveis.**
- Arquivos criptografados podem ser cerca de um terço maiores que o limite de 32 MiB em texto puro por causa do base64; o limite do conteúdo em si não muda.

## Verificação

Os testes em Rust cobrem ida e volta do envelope, senha incorreta, adulteração, parâmetros KDF hostis, salt e nonce novos por arquivo, ida e volta de arquivos criptografados e da sincronização, comportamento da senha armazenada, validação e persistência do agendamento, nomes de snapshot, rejeição de travessia de diretório, ordenação, unicidade dentro do mesmo segundo e retenção. `npm test` verifica a cobertura de ACL dos comandos e os catálogos espelhados.
