# Janela privada

Pressione **Ctrl+Shift+N** para abrir uma janela privada. Ela tem a mesma aparência e o mesmo comportamento de uma janela normal: mesma interface, configurações, atalhos e favoritos. Nada do que acontece nela é mantido depois que ela fecha.

## O que ela mantém

- Favoritos (favoritos e pastas), zoom por site, atalhos de busca, configurações da nova aba e preferências do aplicativo são copiados quando a janela abre. Alterações feitas dentro da janela privada ficam nela.
- Histórico de navegação, cookies, logins, armazenamento local, cache, registros de downloads e abas abertas não são copiados e são removidos quando a janela fecha.
- As credenciais de sincronização de backup nunca são copiadas.

## Como funciona

Cada janela privada é um processo separado do aplicativo, com uma pasta de perfil descartável dentro dos dados locais do aplicativo (`private-sessions`). Ao fechar a janela, a pasta é removida. Se o aplicativo travar, a próxima inicialização remove as pastas que já não estão em uso.

## Compromissos

- Os favoritos são uma cópia feita quando a janela abre. Edições não voltam para a janela normal.
- Cada janela privada usa um processo próprio, então consome mais memória.
- Extensões e logins do perfil normal não ficam disponíveis dentro da janela privada.
- Arquivos baixados em uma janela privada são salvos onde você escolher e não são removidos.
