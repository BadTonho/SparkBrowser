# Widgets da nova guia

Abra **Personalizar nova guia** no canto inferior direito para configurar **Clima**, **Gmail**, **Plano de fundo** e **Relógio**. O painel compacto abre acima do botão, com ícones SVG e um menu vertical de categorias. Seu tamanho responsivo permanece estável ao alternar entre categorias; configurações mais longas rolam dentro do painel. Ao selecionar uma categoria, as opções deslizam da esquerda para a direita; o botão de voltar inverte a transição. O movimento é reduzido quando solicitado pelo sistema operacional. As preferências são locais do dispositivo, compartilhadas entre workspaces e restauradas após reiniciar. O clima fica no canto superior esquerdo e o Gmail no superior direito; painéis estreitos os empilham acima da busca. O diálogo aceita Tab, Escape e clique externo, devolvendo o foco ao botão ao fechar. Rascunhos da busca e sites recentes são preservados durante as atualizações dos widgets.

## Clima

Escolha uma cidade nos resultados da busca, incluindo estado e país, ou selecione **Usar localização atual**. O Windows obtém uma posição pontual após consentimento do sistema; não há localização por IP nem rastreamento contínuo. Coordenadas automáticas ficam em memória. Localização negada ou indisponível mantém a escolha manual disponível. Outras plataformas aceitam seleção manual.

O widget mostra temperatura atual, condição, sensação térmica e três dias de previsão. Celsius é o padrão; Fahrenheit é convertido localmente. As previsões são atualizadas a cada 15 minutos enquanto uma nova guia estiver visível, com um único cache compartilhado em memória. Falhas preservam a última previsão e mostram um erro. **Atualizar** tenta novamente sem recarregar o documento. Não há consultas de clima com o widget desativado.

Previsão e busca de cidades usam o [Open-Meteo](https://open-meteo.com/). Coordenadas são enviadas ao provedor de previsão; nomes de cidades ao serviço de geocodificação. O endpoint gratuito é destinado aqui ao uso pessoal não comercial; [uso comercial exige o plano adequado](https://open-meteo.com/en/pricing). Os ícones são SVGs locais empacotados. A atribuição abre o site do provedor no painel atual.

## Integração com o Gmail

O widget suporta dois modos de conexão:

### 1. Sessão Direta do Navegador (Padrão e Recomendado)
O Veloce configura as WebViews com um User-Agent desktop padrão de navegador, permitindo login direto no Google em abas comuns sem avisos de aplicativo de terceiro. Quando você está conectado ao Gmail (`mail.google.com`) em qualquer aba, o widget detecta automaticamente os cookies da sessão e exibe o número de mensagens não lidas e as mensagens recentes por meio do feed da caixa de entrada. Não é necessário nenhum projeto nem credenciais no Google Cloud Console. Clicar em **Conectar Gmail** abre o `https://mail.google.com/` na guia atual para realizar o login.

### 2. Configuração Desktop OAuth (Opcional / Testes Automatizados)
Para cenários de testes nativos automatizados ou uso de desenvolvedor via OAuth:
1. Crie um projeto no [Google Cloud Console](https://console.cloud.google.com/) e ative a Gmail API.
2. Configure a tela de consentimento OAuth para testes pessoais e adicione sua conta Google como usuário de teste. Adicione somente `https://www.googleapis.com/auth/gmail.metadata`.
3. Crie um cliente OAuth do tipo **Aplicativo para computador** e baixe sua configuração JSON.
4. Coloque o arquivo baixado em `<diretório-de-configuração-do-aplicativo>/oauth/google-desktop.credentials.json`. No Windows, o diretório normal é `%APPDATA%/org.veloce.browser`; desenvolvimento e perfis de teste isolados podem usar outros diretórios. Reinicie o Veloce após adicionar ou alterar esse arquivo.
5. Ative **Mostrar Gmail** e selecione **Conectar Gmail**. A autorização abre o Microsoft Edge fora do Veloce e retorna por um callback temporário em `127.0.0.1`. Conclua o consentimento e volte ao Veloce. A conexão pode ser cancelada pelas configurações.

Nunca coloque a configuração baixada, tokens da conta ou dados de usuário exportados no repositório. O aplicativo lê a configuração padrão `installed` do Google e usa endpoints fixos de autorização/API do Google. Ele não pede a senha Google. Configuração ausente ou armazenamento seguro indisponível produz um estado explicativo, sem simular uma conexão.

O widget aceita uma conta e mostra a quantidade de mensagens não lidas da caixa de entrada e as cinco mais recentes, com remetente, assunto, data e indicação de não lida. Somente metadados são solicitados: corpos, anexos e imagens remotas não são baixados. **Abrir Gmail** navega o painel de origem para a caixa da conta; o Gmail ainda pode exigir um login separado no site. O widget não envia, exclui nem marca mensagens como lidas.

Refresh tokens usam o Gerenciador de Credenciais do Windows, isolados por aplicativo/perfil. Access tokens e resumos da caixa ficam em memória. O e-mail é atualizado a cada 60 segundos enquanto uma nova guia estiver visível; páginas compartilham requisições e falhas aumentam o intervalo. Desativar o widget interrompe consultas e preserva a conexão. **Desconectar** limpa imediatamente o estado local da conta, apaga a credencial salva e tenta revogar o acesso. Acesso expirado ou revogado oferece reconexão.

O modo de testes do Google pode [expirar refresh tokens após sete dias](https://developers.google.com/identity/protocols/oauth2). `gmail.metadata` é um [escopo restrito](https://developers.google.com/workspace/gmail/api/auth/scopes); distribuição pública exige um processo separado de verificação e preparação. Outros provedores e a configuração para distribuição pública ficam fora desta primeira versão. A conexão Gmail exige atualmente armazenamento seguro do Windows e Microsoft Edge.

## Plano de fundo (Imagens e Vídeos)

Você pode definir uma imagem ou vídeo em loop personalizado a partir do seu dispositivo pelo painel **Personalizar nova guia** (`⚙️`).
- **Escolher imagem ou vídeo**: Abre o seletor de arquivos do sistema para escolher uma imagem (`.png`, `.jpg`, `.jpeg`, `.webp`, `.gif`, `.bmp`, `.avif`) ou vídeo (`.mp4`, `.webm`, `.ogg`).
- **Reprodução de vídeo**: Vídeos são reproduzidos automaticamente em loop contínuo, sem som (*muted*) e ajustados para cobrir a tela (`object-fit: cover`) sob a busca e os widgets. Há suporte para arquivos de até 50 MB.
- **Escala e Otimização de imagens**: Imagens de alta resolução são redimensionadas e otimizadas localmente via HTML Canvas para manter a fluidez e reduzir o uso de memória.
- **Escurecimento do fundo**: Um controle deslizante ajusta a camada escura sobre a mídia (0% a 90%, padrão de 40%) para garantir legibilidade dos textos, widgets e atalhos.
- **Efeito translúcido (Glassmorphism)**: Com uma mídia de fundo ativa, os cartões de busca e widgets adotam transparência com desfoque de fundo (*backdrop blur*).
- **Sincronização entre guias e persistência**: A mídia escolhida fica salva localmente no armazenamento do dispositivo (Blobs no IndexedDB e preferências locais) e sincroniza entre todas as novas guias abertas em tempo real, sem vazar para backups.
- **Remover**: Excluir a mídia limpa as URLs de objetos e restaura imediatamente a cor sólida padrão.

## Relógio

O widget de relógio personalizável oferece exibição de horário local e offline na nova guia:
- **Habilitado por Padrão**: Inicia ativo e centralizado na parte inferior da página, abaixo dos atalhos recentes.
- **Opções de Posição**: Selecionáveis pela categoria Relógio nas configurações:
  - **Parte de baixo (centralizado)** (padrão): Exibição ampla posicionada abaixo dos atalhos recentes.
  - **Acima da busca**: Estilo dashboard com hora em destaque acima do formulário de busca.
  - **Cartão de widget**: Cartão retangular na grade superior de widgets ao lado de Clima e Gmail.
- **Seleção de Fonte**: Escolha entre **Padrão do sistema**, **Digital (Monoespaçada)**, **Clássica (Serifada)**, **Moderna arredondada** e **Elegante display**. Todas as fontes utilizam números tabulares (`font-variant-numeric: tabular-nums`) para evitar oscilações na largura dos dígitos.
- **Opções de Formato**: Alternância entre formato 24 horas (`14:30`) e 12 horas (`2:30 PM`).
- **Segundos e Data**: Opções para exibir segundos em tempo real e data por extenso localizada (ex: `quinta-feira, 5 de novembro`).
- **Saudação Dinâmica**: Saudação opcional baseada na hora do dia (*Bom dia*, *Boa tarde*, *Boa noite*).
- **Eficiência**: As atualizações se alinham à virada do próximo minuto quando os segundos estão desativados, eliminando temporizadores desnecessários e poupando bateria. Sincroniza em tempo real entre guias abertas por eventos de armazenamento local.

## Dados e validação

Uma migração SQLite idempotente armazena somente preferências dos widgets e a cidade escolhida manualmente. Backups excluem essas configurações, configuração OAuth, coordenadas, credenciais e e-mails. O documento local mantém `connect-src 'none'`: o Rust faz as consultas remotas e envia dados seguros de apresentação pela ponte autenticada existente. Respostas antigas não restauram widgets desativados nem e-mails desconectados. Abrir configurações cancela prévias de arraste de guias sem ocultar ou recarregar páginas.

Execute `npm run validate` e os cenários isolados `new-tab`, `keyboard`, `split-tab-groups`, `workspace-switcher` e `backup`. Os testes usam fixtures geradas de clima/Gmail/OAuth em loopback com contas artificiais, sem autorização real nem acesso ao cofre de credenciais do usuário. Consentimento real de localização no Windows e autorização com uma conta de teste Google fornecida pelo responsável continuam como verificações manuais. Consulte [testes nativos](../contributing/native-tests.md).
