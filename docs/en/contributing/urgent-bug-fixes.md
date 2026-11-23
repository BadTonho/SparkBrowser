# Urgent Bug Fixes (Hotfixes)

How to ship an urgent fix **without** releasing features that are on `main` but not tested yet.

## Scope

- **Problem solved:** an urgent bug appears while `main` already has features that are not tested. Releasing from `main` would ship them too.
- **Solution:** a hotfix branch created from the last stable tag. This is all that is needed for this case.
- **Not covered here:** feature flags. They are a separate topic (keeping a feature on `main` but switched off in stable builds) and are **not required** to ship an urgent fix. Untested features stay out of the hotfix simply because the branch does not contain them.

## The idea

A tag points to the exact commit that was released. A branch created from that tag contains **nothing** that was done on `main` afterwards, so untested features stay out of the build by construction.

## Steps

Example: the last stable release is `v0.1.5` and the fix will be `0.1.6`.

1. **Create the branch from the last stable tag:**
   ```bash
   git switch -c hotfix/0.1.6 v0.1.5
   ```
2. **Fix the bug in one commit** that contains only the fix.
3. **Prepare the release in a separate commit:** bump the version (`npm run release:version -- v0.1.6`) and add `changelog/0.1.6/en.md` and `pt-BR.md` with only the **Fixes** category (see the [changelog guide](changelog.md)).
4. **Validate and publish:** run the CI/[validation](validation.md) on the branch, create the tag `v0.1.6` on it and follow the [release steps](releases.md).
5. **Bring the fix back to `main` with `cherry-pick`, not a merge:**
   ```bash
   git switch main
   git cherry-pick -x <fix-commit>
   ```
   Also copy the `changelog/0.1.6/` folder to `main` so the history stays complete.

## Rule of thumb

- The **fix commit** goes to both places.
- The **version/release commit** stays only on the hotfix branch. A full merge would carry the `0.1.6` version number into `main`, which may already be heading to another version.

## Tips

- **Keep commits atomic.** If a fix is mixed with a feature in the same commit, the cherry-pick brings both. One commit for the bug, other commits for the features.
- **Fix on `main` first when possible**, then cherry-pick into the hotfix branch with `-x`. The fix then never lives only on the hotfix branch.
- **Expect conflicts when `main` changed the same code.** Resolve them by hand and test the build, because the hotfix branch runs different code from `main`.
- **Name the branch predictably** (`hotfix/<version>`), so it is easy to find later.
- **Test the hotfix build**, not only `main`. The code that ships is the code on the hotfix branch.
- **Never include unfinished work.** Check `git log v0.1.5..hotfix/0.1.6`: it must list only the fix and the release commit.
- Before pushing, verify with `git status`, `git diff` and `.gitignore` that no sensitive data is staged (see the [rules](rules-and-standards.md)).
