---
name: idea2crate-repo-setup
description: Create a new repository's development setup or update an existing repository's AGENTS.md, contribution rules, hooks and CI when a human explicitly requests it. Do not use for ordinary application changes.
---

Use this skill only when a human explicitly asks to create a repository or
update its development rules. Projects own the resulting files; this skill is
an authoring aid, not a synchronization or enforcement service.

## Establish the target

Identify the target directory and whether this is a new project or an existing
one. Read applicable instructions, the working tree status, project documents,
toolchain files, hooks and CI before editing. Use available evidence for language,
platforms, build commands and repository identity. Ask only about consequential
choices that remain unknown; do not invent product scope or executable commands.

For a new project, clarify the intended users, purpose, language and first
verifiable outcome as needed. Create the minimal local scaffold that supports
that outcome using the ecosystem's normal tools. Do not replace a nonempty
directory or assume that a request for local files authorizes remote creation,
publication, paid services or changes to machine-wide configuration.

For an existing project, preserve unrelated work, project-specific requirements,
working checks and existing hook integrations. Inspect actual commands before
documenting them. Resolve conflicting requirements with the user before changing
dependent rules; do not replace an existing AGENTS.md wholesale with a template.

## Prepare project-owned guidance

Read [the baseline](assets/AGENTS.md) and adapt it to the target. It is a starting
point, not a mandatory document or a centrally controlled copy. Preserve its
useful contribution, testing and credential-handling guidance where consistent
with the user's requirements. Make ownership explicit: the project maintains
its own AGENTS.md, hooks and CI, and future skill updates do not change them.

Keep AGENTS.md focused on actionable instructions and verified commands. Link
existing project documentation for product behavior and architecture; do not
create a fixed docs hierarchy when the project already has a suitable one.
Remove obsolete or contradictory wording in the touched scope. Explain any
material change in meaning, especially approvals, tests and publication rules.

If hooks or CI are requested, use the target ecosystem's established tools and
keep the configuration in that repository. Pin external automation to immutable
revisions, preserve security and LFS hooks, and keep credentials out of PR jobs.
The skill itself requires no organization Secret or GitHub App. Do not add a
consumer registry, synchronization lockfile, cross-repository write credentials
or restrictions requiring future project rules to be changed elsewhere.

Project-local installations of central skills may be ignored with
`/.agents/skills/idea2crate-*/`. Preserve tracked project-owned skills; adding an
ignore rule does not remove already tracked files. Do not install this skill
into every new project unless the user requests that dependency.

## Validate and hand over

Review the final diff for lost requirements, conflicting instructions, stale
references and commands unsupported by the repository. Run existing formatting,
lint and checks appropriate to the change. For a new scaffold, run its documented
quick start or build with disposable data. Do not add tests that only assert
template wording or file layout; report unperformed checks honestly.

Show the changed files, material policy choices and validation result. Follow
the target project's normal review and merge process; invocation does not grant
merge, release or remote administration permission. Leave the generated files
independently editable in the target repository. Never schedule an automatic
future update or treat a change in this skill as approval to rewrite projects.
