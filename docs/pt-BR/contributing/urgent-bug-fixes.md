# Correções urgentes de bugs (hotfixes)

Como lançar uma correção urgente **sem** levar para a build as funcionalidades que estão na `main` mas ainda não foram testadas.

## Escopo

- **Problema resolvido:** surge um bug urgente enquanto a `main` já tem funcionalidades que não foram testadas. Lançar a partir da `main` levaria essas funcionalidades junto.
- **Solução:** uma branch de hotfix criada a partir da última tag estável. É só isso que este caso exige.
- **Fora do escopo:** feature flags. É um assunto separado (manter uma funcionalidade na `main` mas desligada nas builds estáveis) e **não é necessário** para lançar uma correção urgente. As funcionalidades não testadas ficam fora do hotfix simplesmente porque a branch não as contém.

## A ideia

Uma tag aponta para o commit exato que foi lançado. Uma branch criada a partir dela **não** contém nada do que foi feito na `main` depois, então as funcionalidades não testadas ficam de fora por construção.

## Passos

Exemplo: a última versão estável é a `v0.1.5` e a correção será a `0.1.6`.

1. **Crie a branch a partir da última tag estável:**
   ```bash
   git switch -c hotfix/0.1.6 v0.1.5
   ```
2. **Faça a correção em um commit** que contenha só o conserto.
3. **Prepare a release em outro commit:** atualize a versão (`npm run release:version -- v0.1.6`) e crie `changelog/0.1.6/en.md` e `pt-BR.md` só com a categoria **Correções** (veja o [guia de changelog](changelog.md)).
4. **Valide e publique:** rode o CI/a [validação](validation.md) na branch, crie a tag `v0.1.6` nela e siga os [passos de release](releases.md).
5. **Leve a correção de volta para a `main` com `cherry-pick`, não com merge:**
   ```bash
   git switch main
   git cherry-pick -x <commit-da-correção>
   ```
   Copie também a pasta `changelog/0.1.6/` para a `main`, para o histórico ficar completo.

## Regra prática

- O **commit da correção** vai para os dois lugares.
- O **commit de versão/release** fica só na branch de hotfix. Um merge completo levaria o número `0.1.6` para a `main`, que talvez já esteja a caminho de outra versão.

## Dicas

- **Faça commits atômicos.** Se a correção vier misturada com uma funcionalidade no mesmo commit, o cherry-pick traz as duas. Um commit para o bug, outros para as funcionalidades.
- **Quando possível, corrija primeiro na `main`** e depois faça cherry-pick com `-x` na branch de hotfix. Assim a correção nunca existe só na branch de hotfix.
- **Espere conflitos se a `main` mexeu no mesmo código.** Resolva manualmente e teste o build, porque a branch de hotfix roda um código diferente da `main`.
- **Dê um nome previsível à branch** (`hotfix/<versão>`), para achá-la depois com facilidade.
- **Teste a build do hotfix**, não só a `main`. O código que vai para os usuários é o da branch de hotfix.
- **Nunca inclua trabalho inacabado.** Confira `git log v0.1.5..hotfix/0.1.6`: só devem aparecer a correção e o commit de release.
- Antes de enviar, confirme com `git status`, `git diff` e `.gitignore` que nenhum dado sensível está preparado (veja as [regras](rules-and-standards.md)).
