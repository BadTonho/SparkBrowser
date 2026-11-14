# Guia de changelog

[English](../../en/contributing/changelog.md)

Toda versão depois da **0.1.0** tem um changelog que conta aos usuários o que mudou. Este guia explica onde os changelogs ficam, como são organizados e como escrevê-los. A regra em si é a [Regra 7](./rules-and-standards.md#regra-7-changelog-em-toda-versão).

A versão **0.1.0** é o lançamento inicial e não tem changelog.

## Estrutura

Os changelogs ficam na pasta `changelog/`, na raiz do repositório, com uma subpasta por versão e um arquivo por idioma:

```text
changelog/
├── README.md
├── 0.2.0/
│   ├── en.md
│   └── pt-BR.md
└── 0.3.0/
    ├── en.md
    └── pt-BR.md
```

- A subpasta tem exatamente o nome da versão no `package.json`, em `src-tauri/Cargo.toml` e em `src-tauri/tauri.conf.json` (por exemplo `0.2.0`). Uma pré-release tem sua própria pasta, com o nome da versão (por exemplo `0.2.0-beta.1`).
- `en.md` é inglês e `pt-BR.md` é português do Brasil. Os dois devem ter sempre os mesmos itens.

## Categorias

Cada arquivo tem três categorias, nesta ordem. Omita a categoria que não tiver itens.

| Português | English | Use para |
| --- | --- | --- |
| **Novas funcionalidades** | **New features** | Algo que o usuário não conseguia fazer antes |
| **Melhorias** | **Improvements** | Um recurso que já existia e agora funciona melhor, mais rápido ou mais claro |
| **Correções** | **Fixes** | Um bug que deixou de acontecer |

## Modelo

`changelog/0.2.0/pt-BR.md`:

```markdown
# Spark 0.2.0

Lançada em: 2026-12-01

## Novas funcionalidades

- Abra janelas adicionais do navegador pelo menu.

## Melhorias

- A hibernação de abas agora mantém ativa a aba que está tocando áudio.

## Correções

- Corrigido o problema em que a lista de downloads não atualizava após um download com falha.
```

`changelog/0.2.0/en.md`:

```markdown
# Spark 0.2.0

Released: 2026-12-01

## New features

- Open additional browser windows from the menu.

## Improvements

- Tab hibernation now keeps the tab that is playing audio awake.

## Fixes

- Fixed the download list not updating after a failed download.
```

## Quando atualizar

Atualize o changelog **na mesma alteração** que muda algo perceptível para o usuário, e não só na hora do lançamento:

1. Encontre a pasta da versão em andamento. Crie-a, com os dois arquivos, se for a primeira mudança dessa versão.
2. Adicione uma linha na categoria certa, **nos dois** arquivos `en.md` e `pt-BR.md`.
3. No dia do lançamento, preencha a data e confira se os dois arquivos estão iguais.

Mudanças que o usuário não percebe não precisam de item: refatoração, testes, documentação interna, CI e ferramentas.

## Como escrever os itens

- Escreva do ponto de vista do usuário, em uma frase curta. Diga o que ele passa a poder fazer ou o que parou de dar errado.
- Use linguagem simples. Evite nomes internos, como módulos, funções ou arquivos, a menos que o usuário os veja.
- Comece com verbo no passado nas correções ("Corrigido…", "Fixed…") e no presente nas funcionalidades e melhorias.
- Cite a plataforma quando a mudança afetar só uma ("No Linux, …", "On Windows, …").
- Nunca reescreva o changelog de uma versão já lançada. Corrija erros na versão seguinte.
