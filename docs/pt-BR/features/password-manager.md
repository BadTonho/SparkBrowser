# Gerenciador de senhas

[English](../../en/features/password-manager.md)

Abra **Configurações → Preenchimento automático e senhas → Gerenciador de senhas** para guardar (a mesma seção tem também [cartões e dados de contato](./autofill.md))  seus logins em um cofre criptografado neste computador. Depois de desbloqueado, ele tem duas páginas, como o gerenciador de senhas de outros navegadores: **Senhas** (busca, **Adicionar** e a lista) e **Configurações** (tudo sobre o próprio cofre).

> **Ainda sem auditoria independente.** O cofre é novo e não foi revisado por um auditor de segurança independente. As Configurações mostram este aviso de forma permanente. Não dependa dele como única cópia de senhas importantes. Veja a [revisão de segurança](../security/account-containers-vault-review.md).

## O que ele faz

- **Criar um cofre** e escolher como ele abre: uma **senha mestra** (de 8 a 256 caracteres), o **Windows Hello** (só no Windows) ou os dois. O Spark não consegue recuperar uma senha mestra por você.
- **As formas de desbloquear são independentes.** Você pode usar só a senha mestra, só o Windows Hello, ou os dois e escolher um a cada desbloqueio. Em **Opções do cofre → Formas de desbloquear** você liga ou desliga cada um depois (desligar a senha mestra pede a senha mais uma vez). Pelo menos uma precisa ficar ligada: a chave de recuperação não conta como uma delas.
- **Windows Hello** pede ao Windows para confirmar que é você (PIN, digital ou rosto) a cada vez. Só funciona neste computador e nesta conta do Windows. Se o cofre abre só com o Windows Hello, redefinir o PIN, trocar de conta do Windows ou reinstalar o sistema pode bloquear essa forma: use a chave de recuperação.
- **Chave de recuperação:** ao criar o cofre você recebe uma chave de recuperação, mostrada **uma única vez**. O Spark não guarda cópia. Ela abre o cofre se você esquecer a senha mestra ou se o Windows Hello deixar de funcionar. Perder todas as formas de entrar, inclusive a chave de recuperação, deixa as senhas salvas ilegíveis. Ao lado de **Copiar chave** há **Baixar chave**, que salva a chave em um arquivo de texto à sua escolha (ele só aceita texto no formato de uma chave de recuperação). O arquivo **não é criptografado**: guarde-o longe deste computador, por exemplo em um pendrive.
- **A lista é agrupada por site.** Todas as contas de um site ficam sob uma linha (com uma letra de identificação e, para várias contas, "N contas"). Clique em uma linha para abri-la (um site com várias contas as lista primeiro). A busca abre todos os sites que combinam.
- **Ver uma senha pergunta quem é você, toda vez.** Clicar em um login pede para confirmar com a **senha mestra** ou o **Windows Hello** (só o Windows Hello, se "Usar o Windows Hello ao mostrar ou copiar senhas" estiver ligado). Quem confere a prova é o backend, não a interface: uma senha mestra errada conta para o freio de tentativas erradas. Depois o login abre em uma página própria: o nome de usuário e a senha (escondida atrás de pontos até você apertar o olho), cada um com um botão de copiar, o site, a observação, e **Editar** e **Excluir**.
- **Encontre um login** com o campo de busca ao lado de **Adicionar senha**. Sites com várias contas mostram a quantidade ao lado do nome. Selecione o site para ver suas contas e depois escolha o login que quer abrir.
- **Ausente por 5 minutos, fecha.** Enquanto uma senha está na tela (ou sendo editada), cinco minutos sem usar a janela, com a aba fora de vista ou com o computador dormindo ou bloqueado a fecham e levam você de volta à lista. É preciso confirmar de novo para vê-la. Isso é separado do bloqueio automático do próprio cofre (1 a 60 minutos), que bloqueia o cofre inteiro.
- **Adicionar, editar, buscar e apagar** entradas com título, site, usuário, senha e notas. O botão **Gerar** cria uma senha aleatória de 20 caracteres (sem caracteres parecidos, como `0`/`O` ou `1`/`l`).
- **Copiar a senha ou o usuário** para a área de transferência. O próprio navegador (não a página de Configurações) esvazia a área de transferência de novo após 30 segundos, mesmo que as Configurações tenham sido fechadas, mas só se ela ainda tiver o que foi copiado: algo que você copiou depois fica intacto. A senha é copiada direto do cofre criptografado, sem passar pela interface. As senhas nunca ficam na lista: só aparecem na página de um login, depois que você confirma quem é, e são copiadas dessa página.
- **Bloquear:** manualmente (**Bloquear agora**), automaticamente após 1, 5 (padrão), 15, 30 ou 60 minutos sem uso, e sempre que o navegador fecha. No cartão de desbloqueio, digite a senha mestra e pressione Enter ou selecione **Desbloquear**. Se o Windows Hello estiver configurado, seu botão aparece abaixo do botão **Desbloquear**, ou como ação principal quando for o único método. Para usar a chave de recuperação, selecione **Usar a chave de recuperação** no rodapé do cartão.
- **Trocar a senha mestra** (a atual é pedida de novo). As entradas salvas, o Windows Hello e a chave de recuperação continuam funcionando.
- **Recomeçar** se as duas senhas forem perdidas: a tela de desbloqueio oferece apagar o cofre. Isso apaga todas as senhas salvas para sempre.

- **Usar o Windows Hello ao mostrar ou copiar senhas** (página Configurações, exige o Windows Hello como forma de desbloquear): o Windows confirma que é você sempre que uma senha é mostrada, copiada ou exportada. Desligar isso pede o Windows Hello mais uma vez, e desligar o Windows Hello como forma de desbloquear também desliga isso.
- **Importar senhas** de um arquivo CSV (o formato que Chrome, Edge e a maioria dos gerenciadores exportam: `name, url, username, password, note`, com outros nomes de coluna comuns também aceitos). O Spark abre o diálogo de arquivo e lê o arquivo ele mesmo, então o conteúdo nunca passa pela interface. Senhas que já estão no cofre (mesmo endereço, usuário e senha) não são adicionadas duas vezes, e linhas sem senha ou com campos longos demais ficam de fora. O resultado é mostrado: adicionadas, já estavam, ficaram de fora. O arquivo CSV continua no disco: exclua-o depois.
- **Exportar senhas** para um arquivo CSV. O Spark primeiro confirma que é você (Windows Hello quando o cofre tem, senão a senha mestra) e depois pergunta onde salvar. **O arquivo não é criptografado**: quem abrir consegue ler todas as senhas, então exclua-o assim que terminar.
- **Excluir todos os dados** apaga o cofre e a chave de recuperação (após uma confirmação), também com o cofre desbloqueado.

Os logins são [preenchidos nas páginas](./autofill.md#preenchendo-nas-páginas) quando você clica em um campo de login. O Spark também [oferece salvar ou atualizar um login](./autofill.md#salvando-o-que-você-digita) ao enviar um formulário.

## Como ele é protegido

- Uma chave aleatória de 256 bits cifra cada entrada (AES-256-GCM, um nonce aleatório novo a cada vez, com o id da entrada preso ao texto cifrado). Títulos, sites, usuários, senhas e notas são todos cifrados; nada legível é guardado.
- Essa chave é guardada uma vez por forma de desbloquear, cada cópia protegida por uma chave própria. A senha mestra e a chave de recuperação passam cada uma por Argon2id (64 MiB, 3 passagens). O Windows Hello funciona de outro jeito: o Windows guarda um par de chaves para o cofre e assina uma mensagem fixa só depois que você confirma com PIN, digital ou rosto. Essa assinatura, que só aquele par de chaves produz, protege a terceira cópia. O Spark não guarda nada secreto para isso. Trocar ou desligar uma forma de desbloquear só reembrulha ou remove aquela cópia, então as entradas não são recifradas.
- Uma falha no desbloqueio com o Windows Hello não conta como tentativa errada (o Windows já controla isso), mas uma senha mestra ou chave de recuperação errada continua contando.
- A chave fica na memória só enquanto o cofre está desbloqueado e é apagada quando ele bloqueia. Ela nunca é gravada em disco.
- Após 3 tentativas erradas, cada nova tentativa precisa esperar, com o atraso dobrando até 60 segundos.
- Só a própria interface do navegador conversa com o cofre; páginas da web não o alcançam. **Janelas privadas** recusam todos os comandos do cofre.
- As tabelas do cofre **não** entram em backups, na sincronização por WebDAV ou Drive, nem em perfis de janelas privadas.
- As falhas são escritas no [log de erros](error-log.md) com o que falhou e onde. O log nunca recebe senhas, chaves de recuperação, conteúdo de entradas nem chaves.

## Limites

- Quem puder executar código como você enquanto o cofre está desbloqueado consegue lê-lo. Bloqueie o cofre ao se afastar.
- Copiar uma senha a deixa disponível a todo programa que lê a área de transferência até ela ser limpa. No Windows a cópia é marcada para ficar fora do histórico e da área de transferência na nuvem, e também é limpa se o navegador fechar antes. No Linux a cópia some quando o navegador fecha, mas um gerenciador de área de transferência pode já ter guardado a própria cópia.
- O Windows Hello só é oferecido no Windows. O Linux ainda não tem equivalente: lá o cofre abre com a senha mestra. A etapa do Windows Hello não foi testada em todo tipo de dispositivo (só PIN, leitor de digital, câmera).
- O Windows Hello prova que alguém que consegue entrar no Windows está presente. Não é um segundo segredo: quem puder executar código como você enquanto o cofre está desbloqueado ainda consegue lê-lo.
- Ainda não há verificação de senhas contra vazamentos conhecidos.
