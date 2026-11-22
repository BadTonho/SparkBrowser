# Spark 0.1.5

## Novas funcionalidades

- Seção **Downloads** nas Configurações: escolha a pasta onde os downloads são salvos (e volte ao padrão), mantenha ou desligue a pergunta de onde salvar cada arquivo e defina se o painel de downloads abre quando um download termina. A pergunta de onde salvar saiu do Desempenho e veio para cá.
- **Preenchimento automático nas páginas:** clique em um campo de login, cartão ou contato e escolha um item salvo para preencher o formulário. Logins só aparecem no próprio site, cartões nunca preenchem o código de segurança e, com o Windows Hello configurado para ser pedido sempre, ele é pedido antes de preencher um login ou cartão. Interruptores para desligar e para permitir em janelas privadas (uma cópia somente leitura do cofre).
- **Salvar o que você digita:** depois de enviar um formulário de login, pagamento ou endereço, o Spark pergunta se deve salvar (ou atualizar uma senha alterada). Só pergunta com o cofre desbloqueado, nunca em janelas privadas, e não repete uma pergunta que você recusou.
- Seção **Preenchimento automático e senhas** nas Configurações, com **Pagamentos** (cartões salvos; o código de segurança nunca é guardado e abrir um cartão pede a senha mestra ou o Windows Hello) e **Dados de contato** (nomes, e-mails, telefones e endereços), ambos guardados no cofre criptografado junto das senhas.
- Arraste uma aba para fora da janela do navegador e solte para abri-la em uma nova janela, posicionada onde você soltou.
- **Perfis:** as Configurações ganharam uma seção Perfis para criar perfis separados, cada um com o próprio histórico, favoritos, cookies, logins, cofre de senhas e configurações, e abri-los em janelas próprias (vários ao mesmo tempo). Dá para renomear e apagar; um perfil diferente do Padrão mostra o nome no título da janela. As atualizações são instaladas pelo perfil Padrão.
- **Códigos 2FA opcionais no gerenciador de senhas:** um login pode guardar o segredo de dois fatores (cole a chave de configuração ou o link `otpauth://` que o site mostra, ou leia o QR code de uma imagem) e o código de seis dígitos aparece junto da senha, com contagem regressiva e botão de copiar. O segredo é cifrado junto com o login e o código é calculado pelo navegador. Logins sem ele continuam iguais.

## Melhorias

- Abrir o Spark de novo com ele já em execução agora abre uma nova janela do navegador na instância aberta, então também dá para criar uma janela por monitor pelo atalho.

## Correções

- Corrigida a falha intermitente ao exibir Configurações quando a aba era aberta ou trocada rapidamente.
- Uma segunda abertura com o mesmo perfil não trava mais e mantém a primeira instância em execução.
- O Spark deixou de registrar falhas de limpeza quando a pasta da atualização preparada ou a chave do Windows Hello já estava ausente.
- O Spark deixou de consultar a mídia da página quando Configurações é a aba ativa.
- Corrigido o botão **Voltar à aba** do vídeo flutuante para fechar a janela e restaurar o vídeo na aba de origem.
