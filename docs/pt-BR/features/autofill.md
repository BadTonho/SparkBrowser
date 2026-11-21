# Preenchimento automático

[English](../../en/features/autofill.md)

Abra **Configurações → Preenchimento automático e senhas**. A seção abre numa lista, como em outros navegadores: **Gerenciador de senhas** (o [cofre de logins](./password-manager.md)), **Pagamentos** e **Dados de contato**.

Tudo aqui fica no mesmo cofre criptografado das senhas. Você o cria e desbloqueia uma vez; o [gerenciador de senhas](./password-manager.md) explica como.

## Pagamentos

Salve cartões para preencher na hora de pagar. Um cartão tem número, nome impresso, mês e ano de validade e um apelido opcional.

- **O código de segurança (CVV/CVC) nunca é salvo.** Você o digita no site a cada vez.
- A lista mostra só o apelido (ou a bandeira) e os **quatro últimos dígitos**, nunca o número inteiro.
- **Abrir um cartão pede que você prove quem é**, toda vez: a senha mestra ou o Windows Hello, como ao abrir uma senha. Quem confere a prova é o backend. Um cartão aberto e sem uso da janela por cinco minutos se fecha sozinho.
- O número precisa ter de 12 a 19 dígitos e passar na verificação do cartão (Luhn); espaços e traços são ignorados. Mês e ano são obrigatórios.
- Até 200 cartões.

## Dados de contato

Salve nomes, e-mails, telefones e endereços. Todos os campos são opcionais, mas ao menos um precisa estar preenchido, e o e-mail precisa parecer um e-mail. Os dados de contato abrem junto com o cofre (sem confirmação extra), porque são bem menos sensíveis que um cartão ou uma senha. Até 200.

## Preenchendo nas páginas

Clique em um campo de login, cartão ou contato e o Spark lista o que combina embaixo dele. Escolha um item com o mouse (ou ↓ e Enter) e o formulário é preenchido.

- **Logins** aparecem em páginas do site para o qual foram salvos (o endereço salvo ou um subdomínio dele), nunca em endereços parecidos, e nunca na versão `http://` simples de um site salvo com `https://`. Clique no campo de usuário ou de senha.
- **Cartões** aparecem nos campos de número, nome e validade. O código de segurança nunca é preenchido: digite você mesmo. Mês e ano preenchem tanto listas (select) quanto campos `MM/AA` ou `MM/AAAA`.
- **Dados de contato** aparecem nos campos de nome, e-mail, telefone e endereço.
- A lista mostra só títulos (um e-mail, `Visa •••• 4242`), nunca uma senha nem um cartão inteiro. Ela fica numa parte fechada da página que o site não consegue ler e só reage a cliques e teclas reais. O que preenche o formulário vem do Spark depois que você escolhe.
- Com **Usar o Windows Hello ao mostrar ou copiar senhas** ligado (veja o [gerenciador de senhas](./password-manager.md)), preencher um login ou um cartão pede o Windows Hello toda vez. Dados de contato nunca pedem.
- **Cofre bloqueado:** a lista avisa. Com o Windows Hello ela oferece **Desbloquear com o Windows Hello**; senão, desbloqueie nas Configurações.
- Caixas de busca, outros quadros dentro de uma página (como iframes de pagamento) e as páginas do próprio Spark não são preenchidos. Só a página em que você está: se você for para outro site antes de escolher, nada é preenchido.
- O texto da lista segue o idioma do seu sistema, não o idioma escolhido no Spark.

### Salvando o que você digita

Envie um cadastro, um login, um pagamento ou um endereço e o Spark pode perguntar, numa barra no canto superior direito da página: **Salvar a senha de example.com?**, **Atualizar a senha de example.com?**, **Salvar o cartão Visa •••• 4242?** ou **Salvar estes dados de contato?**. Nada é salvo até você clicar em **Salvar** (ou **Atualizar**).

- Ele só pergunta com o cofre **desbloqueado**, nunca em janela privada, e não quando o mesmo login ou cartão já está salvo. Uma senha diferente oferece **Atualizar**.
- Um cartão só é oferecido com número válido (checksum) e validade; o código de segurança nunca é lido. Dados de contato precisam de endereço ou telefone e de ao menos três campos preenchidos.
- Se o envio levar a outra página do mesmo site, a pergunta é feita lá (por até 90 segundos).
- **Agora não** impede essa mesma pergunta para aquele site até o Spark reiniciar.

### Os interruptores

Abaixo da lista, nas Configurações:

- **Oferecer preenchimento de formulários** (ligado por padrão) liga ou desliga tudo isso.
- **Permitir preenchimento automático em janelas privadas** (desligado por padrão). Ligado, uma **nova** janela privada leva uma **cópia somente leitura** do cofre criptografado: você o desbloqueia lá (Windows Hello pela lista, ou a senha mestra nas Configurações dela) e pode preencher, mas nunca adicionar, editar nem apagar. A cópia some junto com a janela. Janelas já abertas não mudam.

## Fora do escopo

- **Aliases de e-mail** (endereços que encaminham para o seu e-mail) dependem de um serviço online que o Spark não mantém, então não são oferecidos.

## Testes

```
node tools/testing/native-smoke.mjs --case autofill
node --test tests/autofill-roles.test.mjs
```

`autofill` preenche um login, um cartão e um contato numa página real e confere os casos acima (outro site, caixa de busca, cofre bloqueado, interruptor desligado) e as perguntas de salvar (salvar, já salvo, Agora não). `autofill-roles` cobre como os campos são reconhecidos.

## Armazenamento

Cartões e dados de contato ficam na tabela `vault_records`, criptografados um a um com a chave do cofre (AES-GCM, como os logins). Como o cofre, não entram em backups, na sincronização WebDAV nem no Drive, e não são copiados para perfis de janelas privadas. Detalhes: [gerenciador de senhas](./password-manager.md) e a [revisão de segurança](../security/account-containers-vault-review.md).
