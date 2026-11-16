# Exceções de hibernação automática

## Hibernar outros workspaces

**Configurações → Desempenho → Hibernar outros workspaces** vem **ativada por padrão**. Alguns segundos depois de você trocar de workspace (três, para que alternar entre workspaces com `Alt + 1` … `Alt + 9` não coloque cada um para dormir pelo caminho), as abas dos workspaces em que você não está são hibernadas: as páginas são fechadas para liberar memória e recarregam quando você as abre. Ativar a opção a executa uma vez para o workspace atual.

Ela é independente de **Hibernar abas inativas automaticamente**, então também vale quando essa está **Desativada**; desative esta opção para manter as páginas de todos os workspaces carregadas. Abas tocando mídia ou em Picture-in-Picture, com formulário alterado, baixando, carregando ou fixadas continuam acordadas, assim como os domínios de exceção abaixo. Só as abas da própria janela são afetadas. A preferência é local ao dispositivo e fica fora dos backups.

Para compartilhar ou separar cookies e armazenamento entre workspaces, use **Configurações → Workspaces e tema → Isolar conteúdo por workspace** (**Compartilhado** por padrão, **Isolado** quando ativada); veja [isolamento de workspaces](workspace-isolation.md).

## Exceções de hibernação

**Configurações → Desempenho e privacidade → Exceções de hibernação** mantém os domínios listados e seus subdomínios fora da hibernação automática, inclusive pelo limite de abas residentes. Informe nomes de host sem esquema, caminho ou porta. O navegador remove espaços e o ponto final, normaliza maiúsculas e nomes internacionalizados e compara limites entre rótulos DNS: `exemplo.com` cobre `loja.exemplo.com`, mas não `naoexemplo.com`.

**Desativado significa desativado.** Com **Hibernar abas inativas automaticamente** em **Desativado**, nenhuma aba é colocada para dormir sozinha, inclusive pelo limite de abas residentes. O limite mantém o valor (o campo fica desabilitado) e volta a valer assim que outro modo for escolhido. A hibernação manual continua sempre disponível.

A lista não altera os comandos explícitos de hibernação. O usuário ainda pode hibernar manualmente uma aba isenta; as proteções existentes, como aba ativa e formulário alterado, continuam valendo. Ao remover um domínio, suas abas ficam elegíveis para uma próxima passagem de hibernação automática.

A lista é armazenada nas preferências locais e incluída nos backups. Backups antigos sem a lista restauram uma lista vazia.
