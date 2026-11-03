# Correctness fixes — November 2, 2026

These changes address the five failures reproduced after the optimization review. No commit is created by this workflow.

| Failure | Correction | Regression coverage |
| --- | --- | --- |
| Shield policy changed before a failing session write | On Windows, session persistence succeeds before live policy changes. The fallback engine plans and persists metadata before rebuilding documents. | Native SQLite write rejection preserves policy, document load count and navigation history; successful retries cover both toggle directions. Rust verifies fallback metadata rollback. |
| Workspace deletion committed before activation recording failed | Workspace removal, session tabs, split layouts and activation recording share one SQLite transaction. State is published only after commit. | Native activation rejection preserves the workspace, session, usage records and active tab; retries cover existing and newly created fallback tabs, plus successful tab migration. |
| Failed split replacement destroyed a resident page's history | Plan and persist split/tab metadata before reconciling native WebViews. | Native insertion rejection with a two-view cap preserves the original pair, load counts and back history, followed by a successful retry. |
| Failed ratio save left memory and geometry ahead of SQLite | Keep drag previews separate from committed layouts; restore the committed ratio and native geometry after write failure. | Rust verifies preview isolation and rollback. Native tests measure pane width, unrelated persistence during preview, failed/successful commits and cancellation. |
| A search containing NUL caused an FTS parser error | Route NUL queries through the existing literal `instr` fallback on folded text. | Rust covers notes, session pages, short queries and no-match results; native command search covers saved notes. |

Post-commit native reconciliation errors use the existing visible error event while retaining the committed snapshot, so the UI can represent the saved change. Transaction failures happen before destructive native operations.

Workspace deletion also preserves the destination's split invariant: closing an active workspace returns to the destination's remembered focused pane; moving its active tab into a split replaces only the focused pane. Native regressions include an additional unpaired destination tab to catch accidental selection of a third visible page.

Documentation now includes the missing English performance guide and removes links to review files absent from this checkout. Feature descriptions and the native scenario inventory are synchronized in English and Portuguese.

## Validation

`npm run validate` passed. After the final workspace/split focus adjustment, Rust formatting, Clippy with `-D warnings` and the complete Rust tests passed again. JavaScript/documentation checks and the isolated native build also passed.

| Check | Result |
| --- | --- |
| JavaScript | 65 passed |
| Svelte/TypeScript | 0 errors, 0 warnings |
| Rust unit tests | 81 passed, 1 explicit benchmark ignored |
| Rust updater integration | 1 passed, 1 signed-installer fixture test ignored (requires separate fixture build) |
| Native persistence regressions | Split, workspace and shield scenarios passed on the final isolated Windows build; workspace coverage includes the split destination |
| Native search and feature restoration | NUL note search passed; split/session feature restoration passed, including a final feature rerun |
| PiP FPS | All six ten-second samples passed: approximately 30 FPS and 59.5–59.7 FPS for 30/60 FPS fixtures; maximum gaps at most 53.7 ms, no reported source drops |

The complete native run initially passed 24 of 28 scenarios. A late navigation callback hit the shield test's artificial session-write trigger; the test now acknowledges that expected queued error before retrying, and the shield rerun passed. The main PiP scenario also passed on rerun after an initial foreground-focus interruption. Desktop-dependent focus results are recorded separately below.

All 28 scenarios had passing executions across the complete run and targeted reruns. The shadow scenario passed standalone. Keyboard passed with explicit activation of the isolated window, now included in its setup; a subsequent standalone repetition was interrupted when another browser took foreground focus. Input ownership checks remain enabled for every key. This is not a claim of one uninterrupted 28-scenario green run: foreground focus remained unstable in this desktop session.

Local logs are under the ignored `.cache/` directory: `correctness-validation.log`, `correctness-rust-final.log`, `correctness-native.log`, `correctness-native-recheck.log`, `correctness-native-focus.log`, `correctness-native-keyboard.log` and `correctness-native-shadow.log`. Scenario artifacts retain pass/failure reports; data/profile/config/cache/logs directories are cleaned afterward.

macOS/Linux native fallback execution requires those platforms; its metadata rollback is covered by Rust tests here. These checks do not establish the absence of every possible browser defect.
