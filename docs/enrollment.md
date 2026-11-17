# Enrolling a repository

1. Ensure the App is installed on the new repository with the approved scoped
   permissions. Add the repository, its actual default branch and its pre-commit
   profile to central `repositories.json` through a reviewed central PR.
2. Preserve all existing project instructions in `docs/project.md`, or link from
   that file to detailed product/platform rules. Do not discard them when making
   the root `AGENTS.md` common. Remove nested/override instructions after moving
   their requirements into ordinary docs.
3. From a checkout of the merged central revision, inspect the generated plan:
   `python3 -m standards plan --repository Idea2Crate/PROJECT`.
4. Apply the computed bundle with
   `python3 -m standards apply --repository Idea2Crate/PROJECT --target /path/to/project`.
   Inspect `git diff` and put these files into a migration PR, keeping unrelated
   application work out. Install and run the hooks and existing project CI. The
   command refuses unexpected overrides and symlinks. It replaces an existing
   file only if it is tracked and has no staged or unstaged changes; untracked
   files, including ignored files, cannot be overwritten. An exact no-op does
   not replace anything. Preserve project rules in docs and commit the old
   managed-path files before initial migration. Unrelated work may remain dirty.
5. Record the migration PR's **full head SHA** as `bootstrap_head` and its actual
   enrolled branch SHA as `bootstrap_base` in a reviewed central registry change.
   This approves that exact full migration against that exact baseline, including
   preserved project docs. Both values are required. A changed PR head or enrolled
   base invalidates approval; the grant cannot be replayed after migration.
6. Run central validation, configure its required App check and merge the
   migration. Later policy changes synchronize automatically. Remove the spent
   `bootstrap_head`/`bootstrap_base` pair in a reviewed central change.

The lockfile and generated pre-commit config use the same immutable source SHA.
New source commits may change profiles or retire skills; ordinary PRs may keep
their installed approved source revision within the current format while a sync is pending. Never update a
downstream lockfile alone or accept a contributor-provided digest as approval.

## Explicit format migration

The runtime supports one bundle format, currently version 2. It rejects older
lock formats and source revisions without the current `bundle_version`. There
are no previous-format generators or `legacy` branches in runtime code.

The current two repositories are updated directly through reviewed changes.
First merge the new central implementation. In a clean isolated downstream
checkout, prepare a migration PR from the merged central payload, remove only
the explicitly identified retired shared files and preserve all project skills,
project docs and the project's license. Do not ask `apply` to interpret an older
lockfile; the migration is a one-time patch reviewed as a whole.

Approve the migration head and base using step 5. The checker verifies the new
payload and the exact approved baseline without reading the previous format.
This full-head approval is for enrollment or deliberate migration, not routine
sync; it requires the same final human approval as any central PR. Normal sync
resumes after the migrated baseline is merged. Its job may fail visibly while
waiting for that migration; do not bypass the required App check to merge it.

Inspect any existing `codex/sync-standards` branch during this one-time migration.
If it belongs to a merged App PR and its exact head is unchanged, remove that
obsolete branch so the next sync starts in the current format. Preserve any
unrelated commits or open PR instead of deleting them. Do not leave an old-format
bot branch for the new runtime to interpret.
