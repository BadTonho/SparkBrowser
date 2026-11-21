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

## Fora do escopo

- **Aliases de e-mail** (endereços que encaminham para o seu e-mail) dependem de um serviço online que o Spark não mantém, então não são oferecidos.
- O preenchimento nas páginas e a pergunta "salvar este cartão/endereço?" chegam em etapas seguintes deste recurso; veja o [changelog](../../../changelog/README.md).

## Janelas privadas

O cofre não está disponível em [janelas privadas](./private-window.md).

## Armazenamento

Cartões e dados de contato ficam na tabela `vault_records`, criptografados um a um com a chave do cofre (AES-GCM, como os logins). Como o cofre, não entram em backups, na sincronização WebDAV nem no Drive, e não são copiados para perfis de janelas privadas. Detalhes: [gerenciador de senhas](./password-manager.md) e a [revisão de segurança](../security/account-containers-vault-review.md).
