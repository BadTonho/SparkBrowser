# Janela privada

Abra uma com **Ctrl+Shift+N** ou pelo menu **⋯ Mais** → **Nova janela privada**. Ela tem a mesma interface, as mesmas configurações, atalhos e favoritos de uma janela normal, e nada do que acontece nela é mantido depois que ela fecha.

## Como saber que você está em uma

A janela não tem barra de título nativa, então o próprio Spark a identifica:

- Um selo roxo **Privada**, com um olho riscado, à esquerda da barra de ferramentas, com uma dica que diz o que é apagado ao fechar.
- Um degradê roxo e uma linha roxa na base da barra de ferramentas.
- Na página de nova aba, um aviso (“Você está em uma janela privada”) e um fundo roxo. O aviso também diz os limites: os sites, seu empregador ou seu provedor de internet ainda podem ver o que você faz, e os arquivos que você baixar continuam no seu computador.
- O título da janela no sistema operacional é “Spark (Private)”, que é o que aparece na barra de tarefas e no Alt+Tab.

Uma janela privada não oferece **Nova janela privada** no próprio menu.

## O que ela mantém

- Favoritos (favoritos e pastas), zoom por site, atalhos de busca, configurações da nova aba e preferências do aplicativo são copiados quando a janela abre. Alterações feitas dentro da janela privada ficam nela.
- Histórico de navegação, cookies, logins, armazenamento local, cache, registros de downloads e abas abertas não são copiados e são removidos quando a janela fecha.
- As credenciais de sincronização de backup nunca são copiadas.

## Como funciona

Cada janela privada é um processo separado do aplicativo, com uma pasta de perfil descartável dentro dos dados locais do aplicativo (`private-sessions`). Ao fechar a janela, a pasta é removida. Se o aplicativo travar, a próxima inicialização remove as pastas que já não estão em uso.

## Implementação

- `get_private_window_state` informa à interface se este processo é uma janela privada. A interface então marca o documento (`data-private`), mostra o selo (`PrivateBadge.svelte`) e estiliza a barra de ferramentas. A página de nova aba recebe o mesmo sinal junto com seus outros dados.
- Testes: o aviso da nova aba está coberto em `tests/new-tab.test.mjs`.

## Compromissos

- Os favoritos são uma cópia feita quando a janela abre. Edições não voltam para a janela normal.
- Cada janela privada usa um processo próprio, então consome mais memória.
- Extensões e logins do perfil normal não ficam disponíveis dentro da janela privada.
- Arquivos baixados em uma janela privada são salvos onde você escolher e não são removidos.
