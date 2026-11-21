# Gerenciador de senhas

[English](../../en/features/password-manager.md)

Abra **Configurações → Senhas** para guardar seus logins em um cofre criptografado neste computador.

> **Ainda sem auditoria independente.** O cofre é novo e não foi revisado por um auditor de segurança independente. As Configurações mostram este aviso de forma permanente. Não dependa dele como única cópia de senhas importantes. Veja a [revisão de segurança](../security/account-containers-vault-review.md).

## O que ele faz

- **Criar um cofre** com uma senha mestra (de 8 a 256 caracteres). O Spark não consegue recuperar essa senha por você.
- **Chave de recuperação:** ao criar o cofre você recebe uma chave de recuperação, mostrada **uma única vez**. O Spark não guarda cópia. Ela abre o cofre se você esquecer a senha mestra. Perder a senha e a chave de recuperação deixa as senhas salvas ilegíveis.
- **Adicionar, editar, buscar e apagar** entradas com título, site, usuário, senha e notas. O botão **Gerar** cria uma senha aleatória de 20 caracteres (sem caracteres parecidos, como `0`/`O` ou `1`/`l`).
- **Copiar a senha ou o usuário** para a área de transferência. O próprio navegador (não a página de Configurações) esvazia a área de transferência de novo após 30 segundos, mesmo que as Configurações tenham sido fechadas, mas só se ela ainda tiver o que foi copiado: algo que você copiou depois fica intacto. A senha é copiada direto do cofre criptografado, sem passar pela interface. As senhas só aparecem quando você pede (Mostrar / Editar), nunca na lista.
- **Bloquear:** manualmente (**Bloquear agora**), automaticamente após 1, 5 (padrão), 15, 30 ou 60 minutos sem uso, e sempre que o navegador fecha. Um cofre bloqueado pede a senha mestra ou a chave de recuperação.
- **Trocar a senha mestra** (a atual é pedida de novo). As entradas salvas e a chave de recuperação continuam funcionando.
- **Recomeçar** se as duas senhas forem perdidas: a tela de desbloqueio oferece apagar o cofre. Isso apaga todas as senhas salvas para sempre.

Esta primeira versão só gerencia o cofre nas Configurações. Ela não detecta logins, não oferece salvar senhas nem preenchê-las nas páginas, e não importa nem exporta senhas.

## Como ele é protegido

- Uma chave aleatória de 256 bits cifra cada entrada (AES-256-GCM, um nonce aleatório novo a cada vez, com o id da entrada preso ao texto cifrado). Títulos, sites, usuários, senhas e notas são todos cifrados; nada legível é guardado.
- Essa chave é guardada duas vezes, cada cópia protegida por uma chave derivada com Argon2id (64 MiB, 3 passagens): uma da senha mestra e outra da chave de recuperação. Trocar a senha só reembrulha a chave, então as entradas não são recifradas.
- A chave fica na memória só enquanto o cofre está desbloqueado e é apagada quando ele bloqueia. Ela nunca é gravada em disco.
- Após 3 tentativas erradas, cada nova tentativa precisa esperar, com o atraso dobrando até 60 segundos.
- Só a própria interface do navegador conversa com o cofre; páginas da web não o alcançam. **Janelas privadas** recusam todos os comandos do cofre.
- As tabelas do cofre **não** entram em backups, na sincronização por WebDAV ou Drive, nem em perfis de janelas privadas.
- As falhas são escritas no [log de erros](error-log.md) com o que falhou e onde. O log nunca recebe senhas, chaves de recuperação, conteúdo de entradas nem chaves.

## Limites

- Quem puder executar código como você enquanto o cofre está desbloqueado consegue lê-lo. Bloqueie o cofre ao se afastar.
- Copiar uma senha a deixa disponível a todo programa que lê a área de transferência até ela ser limpa. No Windows a cópia é marcada para ficar fora do histórico e da área de transferência na nuvem, e também é limpa se o navegador fechar antes. No Linux a cópia some quando o navegador fecha, mas um gerenciador de área de transferência pode já ter guardado a própria cópia.
- Ainda não há desbloqueio com a conta do sistema operacional ou biometria, nem verificação de senhas contra vazamentos conhecidos.
