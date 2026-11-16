## Change

Describe the problem, resulting behavior and owning domain. Identify any intentional compatibility changes.

## Regression evidence

List the behavior tests and native scenarios executed, including interactions with session, profiles, split panes, persistence or focus when relevant. Record failures and untested platforms explicitly.

Record `validate:quick`, full validation when Rust/shared behavior changes, and affected native scenarios. A passing source-text assertion alone is not evidence of runtime behavior.

## Review checks

- [ ] Domain ownership and public contracts are preserved.
- [ ] Cross-feature behavior is covered by an appropriate regression test.
- [ ] Local checks and platform limitations are recorded; release tags also require critical native checks.
- [ ] English and pt-BR documentation is synchronized.
- [ ] Working/staged diffs and ignore rules contain no credentials or local user data.
