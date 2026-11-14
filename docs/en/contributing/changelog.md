# Changelog guide

[Português](../../pt-BR/contributing/changelog.md)

Every release after **0.1.0** has a changelog that tells users what changed. This guide explains where changelogs live, how they are organized and how to write them. The rule itself is [Rule 7](./rules-and-standards.md#rule-7-changelog-for-every-release).

Version **0.1.0** is the initial release and has no changelog.

## Structure

Changelogs live in the `changelog/` folder at the root of the repository, with one subfolder per version and one file per language:

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

- The subfolder is named exactly like the version in `package.json`, `src-tauri/Cargo.toml` and `src-tauri/tauri.conf.json` (for example `0.2.0`). A pre-release gets its own folder named after its version (for example `0.2.0-beta.1`).
- `en.md` is English and `pt-BR.md` is Brazilian Portuguese. Both must always have the same entries.

## Categories

Each file has three categories, in this order. Omit a category that has no entries.

| English | Português | Use it for |
| --- | --- | --- |
| **New features** | **Novas funcionalidades** | Something users could not do before |
| **Improvements** | **Melhorias** | A feature that already existed and now works better, faster or more clearly |
| **Fixes** | **Correções** | A bug that no longer happens |

## Template

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

## When to update it

Update the changelog **in the same change** that makes something noticeable to users, not at release time:

1. Find the folder of the version in progress. Create it, with both files, if this is the first change of that version.
2. Add one line to the right category in **both** `en.md` and `pt-BR.md`.
3. On release day, fill in the release date and check that the two files match.

Changes users cannot notice do not need an entry: refactoring, tests, internal documentation, CI and tooling.

## Writing entries

- Write from the user's point of view, in one short sentence. Say what they can now do or what stopped going wrong.
- Use plain language. Avoid internal names such as module, function or file names, unless users see them.
- Start with a verb in the past tense for fixes ("Fixed…", "Corrigido…") and in the present for features and improvements.
- Mention the platform when a change affects only one ("On Linux, …", "No Windows, …").
- Never rewrite the changelog of a version that was already released. Correct mistakes in the next version instead.
