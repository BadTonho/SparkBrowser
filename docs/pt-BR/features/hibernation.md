# Exceções de hibernação automática

**Configurações → Desempenho e privacidade → Exceções de hibernação** mantém os domínios listados e seus subdomínios fora da hibernação automática, inclusive pelo limite de abas residentes. Informe nomes de host sem esquema, caminho ou porta. O navegador remove espaços e o ponto final, normaliza maiúsculas e nomes internacionalizados e compara limites entre rótulos DNS: `exemplo.com` cobre `loja.exemplo.com`, mas não `naoexemplo.com`.

**Desativado significa desativado.** Com **Hibernar abas inativas automaticamente** em **Desativado**, nenhuma aba é colocada para dormir sozinha, inclusive pelo limite de abas residentes. O limite mantém o valor (o campo fica desabilitado) e volta a valer assim que outro modo for escolhido. A hibernação manual continua sempre disponível.

A lista não altera os comandos explícitos de hibernação. O usuário ainda pode hibernar manualmente uma aba isenta; as proteções existentes, como aba ativa e formulário alterado, continuam valendo. Ao remover um domínio, suas abas ficam elegíveis para uma próxima passagem de hibernação automática.

A lista é armazenada nas preferências locais e incluída nos backups. Backups antigos sem a lista restauram uma lista vazia.
