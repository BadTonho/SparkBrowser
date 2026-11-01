# Validation

[Português](../../pt-BR/contributing/validation.md)

## Automated checks

Run from the repository root:

```sh
npm ci
npm test
npm run check
npm run build
cargo test --manifest-path src-tauri/Cargo.toml --offline
cargo clippy --manifest-path src-tauri/Cargo.toml --offline --all-targets -- -D warnings
cargo fmt --manifest-path src-tauri/Cargo.toml --check
npm audit
cargo audit --no-fetch --file src-tauri/Cargo.lock
```

On Windows PowerShell, use `npm.cmd` when execution policy blocks `npm.ps1`. Rust's offline checks require dependencies to have been downloaded previously. Audit coverage and unresolved upstream warnings are explained in [dependency notes](../security/dependency-notes.md).

Regression tests cover session rollback after an insertion error and participation in an outer transaction; workspace/tab selection; the selected search engine; tracker matching, URL/Request inputs, XHR reuse and beacon requests; the vendored dependency security fix; and real local HTTP transfers, failed or interrupted responses, redirect policy, and filename collisions. Transfer tests use loopback HTTP and temporary directories rather than the user's Downloads directory.

## Native smoke checks

These checks require an interactive desktop and are separate from unit/build results:

1. Open a page with an editable field, enter text, open a second tab, then return. The text and the first tab's back/forward history should remain independent.
2. Hibernate the first tab while it is inactive, then select it. Only that page should reload. Closing a different background tab must not wake hibernated pages.
3. Select an existing tab from another workspace through the Command Bar. Its workspace button, theme, and tab list should agree. Moving the active tab to another workspace should keep the same invariant.
4. Choose DuckDuckGo, then search through both Command Bar actions: current tab and new tab. Both should use DuckDuckGo.
5. Download a direct HTTP resource. Verify bytes on disk, collision-safe filenames, and an error for a failed response. The preview in a normal browser must not fabricate completed downloads.
6. Change privacy protection on Windows and verify that forms and back/forward history remain intact. Test requests on both current and newly navigated documents. On other engines, verify the documented reload fallback and refusal while detected forms/media/downloads are protected.
7. Navigate to a blocked host. The last committed page URL should remain unchanged. A successful ordinary navigation should add one history entry.

Finish by checking `git status`, `git diff`, `git diff --cached`, and sensitive-file ignore rules.
