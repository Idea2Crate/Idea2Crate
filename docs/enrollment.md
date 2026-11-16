# Enrolling a repository

1. Add the repository, its actual default branch and its pre-commit profile to
   central `repositories.json`. Review this through a central PR.
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
   command refuses edited managed files, unexpected overrides and symlinks.
5. Record the migration PR's **full head SHA** as `bootstrap_head` for this
   repository in a reviewed central registry change. This permits that exact
   onboarding commit, including preserved project docs, while the target has no
   lockfile. Subsequent commits invalidate approval. Bootstrap cannot authorize
   policy changes after the first lockfile is merged.
6. Run central validation, configure its required App check and merge the
   migration. Later policy changes synchronize automatically. Remove the spent
   `bootstrap_head` in a routine central change; it cannot authorize another PR.

The lockfile and generated pre-commit config use the same immutable source SHA.
New source commits may change profiles or retire skills; ordinary PRs may keep
their installed approved version while a sync is pending. Never update a
downstream lockfile alone or accept a contributor-provided digest as approval.
