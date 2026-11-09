# Correções de funcionamento e privacidade

Os sete achados da análise do projeto receberam alterações de código e cobertura de regressão.

- Credenciais WebDAV usam o Gerenciador de Credenciais do Windows ou o Secret Service do Linux por meio de `secret-tool`. O texto simples antigo é removido com exclusão segura do SQLite e checkpoint que trunca o WAL. Se a migração para o cofre falhar, a credencial permanece somente na memória do processo. Cópias externas existentes do banco ficam fora dessa migração.
- A sincronização com o Google Drive vincula o perfil persistente e o índice de conta escolhidos explicitamente. As transferências verificam o e-mail detectado, rejeitam mudanças de identidade e não recorrem a abas descartáveis ou outros perfis de cookies. Usuários existentes precisam selecionar sua conta Google uma vez após atualizar.
- A importação de backup preserva abas descartáveis, ordem local, seleção, participação em divisão e estado dos documentos residentes. A transação de importação preserva as identidades dos containers existentes; abas importadas continuam usando a identidade padrão, conforme documentado.
- Nomes de pastas de workspace preservam os nomes antigos comuns em minúsculas e usam hash para identificadores com maiúsculas, inseguros ou com nomes de dispositivos do Windows, sem colisões por capitalização. Identificadores antigos incomuns podem exigir novo login nos sites.
- A leitura RSS/Atom aceita títulos e links em CDATA, preserva texto aninhado e seleciona alternativas de artigos em vez de links self/API/anexos.
- A formatação Rust e o Clippy estrito passam. Opções de perfil de aba e de cadastro de download substituem listas longas de argumentos posicionais.

As melhorias adicionais estão incluídas: testes de regressão exercitam o comportamento, atualizações de feeds reutilizam um cliente HTTP com quatro trabalhadores e deduplicação por feed, e a integração do Drive separa autenticação, acesso a cookies, transferência HTTP e vínculo de sessão em `src-tauri/src/backup/gdrive/`.

## Verificação

- `npm run validate`: testes JavaScript, checagens Svelte/TypeScript, formatação Rust, Clippy estrito, testes Rust, build frontend e verificações de segurança do repositório.
- Cenários nativos no Windows: `backup-identities` (incluindo reinício forçado imediato), `backup` (incluindo nova tentativa de aplicação de preferências) e `feeds` (incluindo CDATA, alternativas Atom, XML inválido e persistência após reinício).
- Testes de migração inspecionam o banco SQLite em disco e seu WAL em busca da credencial artificial após a migração. Testes do cofre usam um adaptador isolado em memória e nunca acessam credenciais pessoais.

O comportamento nativo no Linux, o adaptador de Secret Service e transferências com uma conta Google real autenticada não foram exercitados neste ambiente. A integração com o Google ainda depende dos endpoints usados pela interface web do serviço; resultados com fixtures nativas não comprovam compatibilidade com futuras alterações do servidor. Consulte os guias de [backup](../../features/backup.md), [isolamento de workspace](../../features/workspace-isolation.md) e [leitor de feeds](../../features/feed-reader.md) para uso e requisitos por plataforma.
