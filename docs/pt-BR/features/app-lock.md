# Bloqueio do app

[English](../../en/features/app-lock.md)

O bloqueio do app pede um segredo toda vez que um [perfil](./profiles.md) abre, para que ninguém use o Spark daquele perfil sem ele. Cada perfil tem o próprio bloqueio, que começa desligado até você ligá-lo. Abra **Configurações → Perfis → Bloqueio do app** para gerenciar o bloqueio do perfil que você está usando.

> **Ele só impede o acesso ao Spark. Não criptografa nada.** O histórico, os favoritos e os demais dados do perfil continuam legíveis para quem abrir a pasta de dados deste computador ou copiá-la. Use-o para impedir que outras pessoas sentem no seu computador aberto e usem o seu navegador, não como proteção contra quem consegue ler os seus arquivos. O Spark diz isso nas Configurações e na tela de bloqueio.

## Formas de abrir um perfil bloqueado

Você pode ligar uma ou mais destas, e qualquer uma abre o perfil:

- **Uma senha própria** (de 8 a 256 caracteres), separada de todas as outras senhas.
- **Windows Hello** (só no Windows): PIN, digital ou rosto. Ele tem uma chave própria, separada da do cofre de senhas.
- **A senha mestra do cofre de senhas do perfil.** Só funciona quando o cofre tem senha mestra. O Spark a confere sem abrir o cofre, então desbloquear o app não desbloqueia o cofre.

Pelo menos uma forma precisa continuar ligada enquanto o bloqueio estiver ligado. Se você depender só do Windows Hello, redefinir o PIN do Windows ou usar outra conta do Windows pode deixar você de fora, então as Configurações sugerem adicionar também uma senha.

## Ligar, alterar e desligar

- **Ligar** digitando uma senha nova e selecionando **Bloquear com uma senha**, ou escolhendo o Windows Hello ou a senha mestra do cofre como primeira forma.
- **Alterar ou adicionar uma forma** (uma senha nova, o Windows Hello, a senha mestra do cofre) e **desligar uma forma** ou **desligar o bloqueio**: o Spark pede que você confirme com uma forma que funciona agora, mesmo com o perfil já aberto. Não dá para desligar a última forma; desligue o bloqueio.
- Desligar a senha mestra do cofre, ou apagar o cofre, é recusado enquanto o bloqueio abrir **só** com essa senha: você ficaria trancado fora do seu próprio navegador. Adicione outra forma antes.

## O que a tela de bloqueio faz

Quando um perfil com bloqueio inicia, o Spark mostra só a tela de bloqueio. **Nenhuma aba, página ou janela do perfil existe por trás dela**, e nenhum comando do navegador funciona até o perfil abrir. Uma segunda abertura do mesmo perfil só traz a tela de bloqueio para a frente. Fechar a janela só encerra o processo, sem salvar nada.

- **Segredos errados são desacelerados.** Depois de 3 tentativas erradas, cada nova tentativa precisa esperar, com a espera dobrando até um minuto, mesmo para o segredo certo. A espera é salva, então reiniciar o Spark não a zera.
- **Esqueceu o segredo?** O bloqueio não criptografa nada, então não dá para removê-lo sem apagar o que está por trás, senão qualquer um poderia simplesmente removê-lo. A tela de bloqueio oferece **Apagar o perfil**, que pede confirmação e depois **apaga para sempre o histórico, os favoritos, os cookies, os logins, as senhas e as configurações daquele perfil deste computador** e o abre de novo, vazio e sem bloqueio. Os outros perfis não são tocados.

## Bom saber

- O bloqueio pertence a um perfil e **não** entra em backups, na sincronização WebDAV ou do Drive, e uma janela privada não o leva. Um backup restaurado nunca traz um bloqueio e nunca remove um.
- Os segredos nunca são guardados: o Spark guarda um hash da senha (esticada com o mesmo Argon2id do cofre de senhas) e um hash da assinatura que o Windows Hello produz.
- Um perfil aberto não bloqueia de novo sozinho: o bloqueio é pedido quando o perfil inicia.
- Toda falha é gravada no [log de erros](error-log.md) (área `app-lock`) sem nenhum segredo.

## Como funciona (para desenvolvedores)

- O bloqueio é uma linha na tabela `app_lock` do banco do perfil: os verificadores, quais formas estão ligadas e o freio contra tentativas erradas (as falhas e o instante antes do qual nenhuma tentativa é aceita). `src-tauri/src/app_lock/` tem a lógica (`ops.rs` trabalha sobre uma conexão e recebe o tempo como argumento; `verifier.rs` os hashes; `erase.rs` o auxiliar que apaga um perfil).
- Na inicialização, com o banco aberto, um perfil com bloqueio liga `app_locked` e para antes do último passo. Esse passo (`session_start::start`: a visão de conteúdo, as janelas da última sessão, as atualizações das listas de filtros, os feeds e o momento em que o navegador fica pronto) só roda quando o perfil não tem bloqueio ou quando o bloqueio é aberto. Como `startup_ready` continua falso, todo comando é recusado, menos os poucos que desbloqueiam, saem ou apagam (`commands::access::answers_before_ready`).
- Esquecer o segredo inicia um processo auxiliar (`--erase-profile <id>`) e encerra este. O auxiliar espera o bloqueio do perfil, apaga os dados (nunca os outros perfis, a escolha do seletor de perfis nem os nomes dos perfis) e inicia o perfil de novo.
- O freio (`src-tauri/src/brake.rs`) é compartilhado com o cofre de senhas; só o bloqueio do app o persiste.
