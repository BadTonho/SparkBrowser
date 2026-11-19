# Regression prevention workflow

Before changing shared behavior, identify its owning domain and write a behavior test that crosses the affected boundary. Separate mechanical moves from behavior changes in the review description. Preserve IPC payloads, backup compatibility, persistent profiles and Windows/Linux support.

## Local validation

- `npm run architecture:check` rejects static runtime import cycles in frontend services/stores, private browser state imports, unauthorized session mutation imports and bypasses of the content factory boundary. Rust visibility also prevents mutable session guards outside the state module.
- `npm run validate` includes architecture checks, JavaScript contracts, Svelte checks, Rust formatting, strict Clippy, Rust tests, frontend build and repository security checks.
- Build the isolated application with `npm run test:native:build:fast`, then run `npm run test:native:critical`. Use `npm run test:native` for the complete suite. See [native setup](native-tests.md).

The critical suite explicitly contains `backup-identities`, `persistence-split`, `persistence-workspaces`, `workspace-switcher`, `recovery`, `shutdown` and `security-boundaries`. Each scenario uses its own profile and loopback fixtures. The backup scenario combines containers, disposable tabs, split layout, a resident form and crash restart. Persistence scenarios inject database failures and retry operations without discarding resident documents.

Contract tests also exercise out-of-order observations, failed workspace movement, drag cancellation after tab closure, disposal during initialization, shared subscriptions and competing panel openings. Existing protection tests cover forms, media, downloads, pinned tabs and visible panes. Native permission scenarios cover request cancellation and remembered decisions; do not invent a new hibernation policy as part of a refactor.

Behavior tests also complete delayed close replies after a workspace switch and retry failed reorders while preserving newer navigation. Keep source inspections for structural contracts such as ACL registration and module boundaries; they do not prove runtime behavior. Rust already exercises workspace-folder isolation and replay rejection, so preserve those executable tests rather than replacing them with text matching.

## Continuous integration and releases

The regular workflow retains Windows and Linux validation. To save GitHub Actions minutes, both the regular workflow and the native suite run **manually only** (Actions tab or `gh workflow run ci.yml` / `gh workflow run native.yml`); there are no runs on push, pull request or schedule. Run them before releases and larger changes. The release workflow does not run the native suite; run `npm run test:native:critical` locally before tagging (see [releases](releases.md)). The scheduled GitHub runs of the full suite failed every time because the application could not start on the runner, so it is not used as a gate. Failures fail the workflow without automatic retries or silent exclusions. Platform/driver prerequisites must be repaired when they fail; do not turn required scenarios into optional checks.

The runner prints diagnostics locally and records names, statuses and durations in `.cache/native-results.json`. CI uploads only this reviewed summary for seven days. Profiles, databases, raw logs and screenshots are excluded from published artifacts. Local isolated scenario artifacts remain available under `.cache/native-runs/` according to the existing harness cleanup policy.

The Linux native driver path is preserved but has not been validated in this Windows environment. Desktop focus, pointer capture, real device/OS prompts and authenticated external accounts retain the limitations described in native testing documentation. A CI workflow definition is not evidence that a hosted runner has passed.

Before delivery, inspect `git status`, working and staged diffs, ignore rules and secret checks. New documentation must have matching English and pt-BR paths. Record the checks actually executed and any failures or platform limitations in the review.
