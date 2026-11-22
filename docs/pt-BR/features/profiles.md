# Perfis

[English](../../en/features/profiles.md)

Um perfil é uma cópia separada de tudo o que o Spark guarda para você: histórico, favoritos, cookies e dados de sites, logins, o [cofre de senhas](./password-manager.md), a lista de downloads e as configurações. Nada é compartilhado entre perfis. Use-os para separar a navegação de trabalho da pessoal, ou para duas pessoas dividirem a mesma conta do computador.

Abra **Configurações → Perfis** para gerenciá-los.

## O que você pode fazer

- **Ver seus perfis.** A lista mostra todos os perfis e marca o que esta janela usa (**Em uso**). O perfil que o Spark sempre usou se chama **Padrão**.
- **Criar** um perfil digitando um nome e selecionando **Criar perfil**. Um perfil novo começa vazio. Os nomes têm de 1 a 40 caracteres, sem quebras de linha, e não podem se repetir (maiúsculas e minúsculas contam como iguais). Você pode ter até 10 perfis.
- **Abrir** um perfil na própria janela. Cada perfil roda como um processo próprio do Spark, então você pode ter vários abertos ao mesmo tempo. Abrir um que já está aberto traz uma nova janela dele em vez de iniciá-lo duas vezes.
- **Renomear** um perfil. O perfil Padrão não pode ser renomeado.
- **Apagar** um perfil. O Spark pede que você digite o nome dele e depois apaga o histórico, os favoritos, os cookies, os logins, as senhas e as configurações **deste computador, para sempre**. Não dá para apagar o perfil Padrão, o perfil que você está usando, nem um perfil aberto em outra janela (feche-o primeiro).

As janelas de um perfil diferente do Padrão mostram o nome do perfil no título ("Spark - Trabalho").

## Bom saber

- **As atualizações são instaladas pelo perfil Padrão.** Dois perfis abertos ao mesmo tempo não podem rodar o mesmo instalador, então os outros perfis não instalam atualizações; a página **Sobre e atualizações** deles diz que as atualizações não estão disponíveis ali. Abra o perfil Padrão de vez em quando para manter o Spark atualizado.
- **Cada perfil tem o próprio cofre de senhas** e a própria chave do Windows Hello para ele, então desbloquear o cofre de um perfil nunca abre o de outro.
- **Backups, sincronização WebDAV/Drive e janelas privadas pertencem ao perfil em que você está.** Um backup contém só os dados daquele perfil.
- **Janelas privadas não são perfis.** São descartáveis e copiam favoritos e configurações do perfil de onde foram abertas. Não dá para gerenciar perfis de uma janela privada.
- O perfil Padrão continua na mesma pasta de antes, então nada se move nem muda para você ao atualizar.

## Onde ficam os dados

O perfil Padrão continua onde o Spark sempre guardou os dados. Cada outro perfil tem a própria pasta dentro da pasta de dados locais do perfil Padrão, em `profiles/<id>/`, com banco de dados, logs, cache e dados do motor web próprios. Cada pasta tem um pequeno `profile.json` com o nome do perfil. O Spark encontra os perfis olhando essas pastas, então nenhum arquivo compartilhado pode ser danificado por dois perfis rodando ao mesmo tempo.

## Como funciona (para desenvolvedores)

- Um perfil diferente do Padrão inicia com `--profile <id>`. Antes de existir qualquer janela, o processo redireciona todos os diretórios do aplicativo (config, dados, dados locais, cache, logs) para a pasta do perfil, o mesmo mecanismo das janelas privadas, então os dados do WebView2 ou do WebKit também ficam isolados. O id tem 12 caracteres hexadecimais minúsculos gerados pelo Spark e é validado antes de chegar a qualquer caminho.
- A pasta de dados de cada perfil tem o próprio `profile.lock`. Abrir um perfil pede ao processo em execução (pelo vínculo de instância) uma nova janela e só inicia um novo processo quando não há nenhum rodando. Apagar usa o mesmo bloqueio, e é assim que um perfil em execução é recusado.
- As falhas são gravadas no [log de erros](error-log.md) com o que falhou e onde (área `profiles`).
- O que era global precisou virar por perfil: a chave do Windows Hello do cofre (nomeada com o perfil; o perfil Padrão mantém o nome que sempre teve) e o atualizador (só o Padrão). As credenciais de backup já eram por perfil, porque a chave delas vem do caminho do banco de dados.
