# Widgets da nova guia

Abra **Personalizar nova guia** no canto inferior direito para ativar **Clima** e **Gmail**. Ambos começam desativados. As preferências são locais do dispositivo, compartilhadas entre workspaces e restauradas após reiniciar. O clima fica no canto superior esquerdo e o Gmail no superior direito; painéis estreitos os empilham acima da busca. Janelas pequenas permitem rolagem. O diálogo de configurações aceita Tab, Escape e clique externo, devolvendo o foco ao botão ao fechar. Rascunhos da busca e sites recentes são preservados durante atualizações.

## Clima

Escolha uma cidade nos resultados da busca, incluindo estado e país, ou selecione **Usar localização atual**. O Windows obtém uma posição pontual após consentimento do sistema; não há localização por IP nem rastreamento contínuo. Coordenadas automáticas ficam em memória. Localização negada ou indisponível mantém a escolha manual disponível. Outras plataformas aceitam seleção manual.

O widget mostra temperatura atual, condição, sensação térmica e três dias de previsão. Celsius é o padrão; Fahrenheit é convertido localmente. As previsões são atualizadas a cada 15 minutos enquanto uma nova guia estiver visível, com um único cache compartilhado em memória. Falhas preservam a última previsão e mostram um erro. **Atualizar** tenta novamente sem recarregar o documento. Não há consultas de clima com o widget desativado.

Previsão e busca de cidades usam o [Open-Meteo](https://open-meteo.com/). Coordenadas são enviadas ao provedor de previsão; nomes de cidades ao serviço de geocodificação. O endpoint gratuito é destinado aqui ao uso pessoal não comercial; [uso comercial exige o plano adequado](https://open-meteo.com/en/pricing). Os ícones são SVGs locais empacotados. A atribuição abre o site do provedor no painel atual.

## Configuração do Gmail para testes pessoais

1. Crie um projeto no [Google Cloud Console](https://console.cloud.google.com/) e ative a Gmail API.
2. Configure a tela de consentimento OAuth para testes pessoais e adicione sua conta Google como usuário de teste. Adicione somente `https://www.googleapis.com/auth/gmail.metadata`.
3. Crie um cliente OAuth do tipo **Aplicativo para computador** e baixe sua configuração JSON.
4. Coloque o arquivo baixado em `<diretório-de-configuração-do-aplicativo>/oauth/google-desktop.credentials.json`. No Windows, o diretório normal é `%APPDATA%/org.veloce.browser`; desenvolvimento e perfis de teste isolados podem usar outros diretórios. Reinicie o Veloce após adicionar ou alterar esse arquivo.
5. Ative **Mostrar Gmail** e selecione **Conectar Gmail**. A autorização abre o Microsoft Edge fora do Veloce e retorna por um callback temporário em `127.0.0.1`. Conclua o consentimento e volte ao Veloce. A conexão pode ser cancelada pelas configurações.

Nunca coloque a configuração baixada, tokens da conta ou dados de usuário exportados no repositório. O aplicativo lê a configuração padrão `installed` do Google e usa endpoints fixos de autorização/API do Google. Ele não pede a senha Google. Configuração ausente ou armazenamento seguro indisponível produz um estado explicativo, sem simular uma conexão.

O widget aceita uma conta e mostra a quantidade de mensagens não lidas da caixa de entrada e as cinco mais recentes, com remetente, assunto, data e indicação de não lida. Somente metadados são solicitados: corpos, anexos e imagens remotas não são baixados. **Abrir Gmail** navega o painel de origem para a caixa da conta; o Gmail ainda pode exigir um login separado no site. O widget não envia, exclui nem marca mensagens como lidas.

Refresh tokens usam o Gerenciador de Credenciais do Windows, isolados por aplicativo/perfil. Access tokens e resumos da caixa ficam em memória. O e-mail é atualizado a cada 60 segundos enquanto uma nova guia estiver visível; páginas compartilham requisições e falhas aumentam o intervalo. Desativar o widget interrompe consultas e preserva a conexão. **Desconectar** limpa imediatamente o estado local da conta, apaga a credencial salva e tenta revogar o acesso. Acesso expirado ou revogado oferece reconexão.

O modo de testes do Google pode [expirar refresh tokens após sete dias](https://developers.google.com/identity/protocols/oauth2). `gmail.metadata` é um [escopo restrito](https://developers.google.com/workspace/gmail/api/auth/scopes); distribuição pública exige um processo separado de verificação e preparação. Outros provedores e a configuração para distribuição pública ficam fora desta primeira versão. A conexão Gmail exige atualmente armazenamento seguro do Windows e Microsoft Edge.

## Dados e validação

Uma migração SQLite idempotente armazena somente preferências dos widgets e a cidade escolhida manualmente. Backups continuam na versão 4 e excluem essas configurações, configuração OAuth, coordenadas, credenciais e e-mails. O documento local mantém `connect-src 'none'`: o Rust faz as consultas remotas e envia dados seguros de apresentação pela ponte autenticada existente. Respostas antigas não restauram widgets desativados nem e-mails desconectados. Abrir configurações cancela prévias de arraste de guias sem ocultar ou recarregar páginas.

Execute `npm run validate` e os cenários isolados `new-tab`, `keyboard`, `split-tab-groups`, `workspace-switcher` e `backup`. Os testes usam fixtures geradas de clima/Gmail/OAuth em loopback com contas artificiais, sem autorização real nem acesso ao cofre de credenciais do usuário. Consentimento real de localização no Windows e autorização com uma conta de teste Google fornecida pelo responsável continuam como verificações manuais. Consulte [testes nativos](../contributing/native-tests.md).
