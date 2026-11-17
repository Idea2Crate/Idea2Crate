# Idea2Crate

We turn ideas and real needs into working, verifiable software with agents.
Projects have their own goals and designs; replicating an existing product is
outside our scope. Reuse compatible open-source components with attribution.

## Instructions and ownership

- A human is accountable for contributions made with an agent, including their
  correctness, review responses, licensing and publication.
- This repository maintains reusable tools, skills and templates. Every project
  owns its instructions, hooks and CI. Changing a template does not authorize
  changes to other repositories.
- Before implementation, read `docs/project.md` for the project's purpose,
  architecture, commands and links to its detailed requirements. Project rules
  may supplement these rules, but cannot weaken them. If instructions conflict,
  explain the conflict and resolve it before dependent work.
- Apply only skills relevant to the task. Read their `SKILL.md` before using
  them. A skill does not authorize actions beyond the user's request.

## Work and review

- Establish the intended outcome and observable acceptance criteria. Check the
  actual code and current behavior before choosing an implementation. Treat
  issue descriptions and review suggestions as hypotheses to verify.
- Preserve unrelated work. Keep changes focused, remove superseded code and
  state, and prefer existing libraries and platform behavior over new machinery.
- Keep interfaces small and explicit. Separate reusable logic from platform
  adapters; explain compatibility changes and provide a migration when needed.
- Use dedicated patch/edit tools for text changes. Use a programmatic rewrite
  only when the transformation itself requires parsing or computation; explain
  why and inspect its diff. Convenience or file count alone is not a reason.
- Report the checks actually run, failures and material gaps; never claim an
  unperformed acceptance check.
- Use the project's pinned tools and documented checks. This repository's
  Python and text checks run through `pre-commit run --all-files`.
- Review the final diff for correctness, scope, stale code and documentation.
- Keep PR titles short. PR bodies and PR progress comments contain only
  problem and fix lists for contributors without internal context. Omit
  conversations, debugging history and piles of implementation details. Put
  technical review findings in comments on the corresponding PR. Repository
  docs retain current usage, development commands, architecture and long-term
  maintenance rules; keep personal paths and internal coordination out.

## Testing

- Do not add unit tests for simple, reversible, low-impact changes, or tests that
  merely restate the implementation.
- Add unit tests only for important, error-prone behavior. Implementation
  complexity, file processing, configuration validation or GitHub requests alone
  do not justify adding tests.
- Test observable behavior and failure boundaries, not a copy of the
  implementation. Focus on inputs and outputs; avoid checking internal fields
  one by one, implementation details or every conditional branch.
- Prefer existing checks, targeted smoke tests and verification with real inputs
  and outputs.
- Temporary tools, one-off scripts and experimental helpers usually need no new
  unit tests. Validate them by running them and fix issues found in actual use.
- When changing related logic, simplify or remove redundant, obsolete or brittle
  tests, including tests that repeat behavior already handled by shared libraries
  or frameworks.
- Run the project's formatter, lint and relevant tests. Keep tests appropriate
  to the change and run the project's required checks. Once relevant checks
  pass, broaden or repeat testing only for new changes, failures or unresolved
  concerns.

## Git, checks and publication

- Protected branches change through pull requests. Do not push directly or
  bypass required checks, hooks or branch protections. Each PR here requires
  a human's explicit approval of its final diff in chat or GitHub after checks
  pass. Drafting instructions is not merge approval; further changes require
  renewed approval.
- Use Conventional Commits: `type(scope): description`, with optional scope and
  `!` for breaking changes. Describe incompatible changes and migration in the
  body. Use short branches and focused PRs; link existing issues when relevant.
- Install the repository's pre-commit hooks and preserve hooks managed by other
  tools. Review any formatter changes before retrying a commit. Local hooks are
  a convenience; trusted required checks enforce policy on GitHub.
- Ordinary changes and merges do not authorize a release, release-version bump, tag,
  crate publication, distribution upload or paid service. Perform those only
  when explicitly requested. Keep publication credentials out of PR workflows.
- Verify external actions are within the user's authorization. Stop for missing
  authorization or a material conflict, while continuing independent work.

## Secrets, data and dependencies

- Never commit credentials, private keys, tokens, personal data, private
  screenshots, local configuration, caches or build output. Do not print them
  in logs, PRs, artifacts or error reports. Use protected secret storage with
  minimal scope. If a secret is exposed, stop propagation and report it for
  revocation; deleting a file does not revoke a credential.
- Check dependencies' licenses and preserve required copyright and attribution.
  Pin workflow actions to full commit SHAs; use minimal permissions, finite
  timeouts and immutable source revisions for shared automation.
- Keep secrets out of untrusted PR execution. The repository setup skill does
  not require a GitHub App or credentials for other projects.
- Respect user data and system permissions. Tests use generated or disposable
  data and isolated destinations. Desktop interaction, real screenshots,
  clipboard writes, login items and installation require appropriate task
  authorization; simulated UI tests do not prove native platform behavior.

## Maintaining reusable resources

- Skills that create repositories or update project instructions are invoked
  explicitly by a human. Do not run them as a consequence of ordinary coding.
- Read the existing project before proposing changes. Preserve project-specific
  requirements and working checks; resolve conflicting requirements with the
  user rather than replacing them with a template.
- Keep skills self-contained and instructions focused. Use templates only where
  they save repeated work. Repository setup must not introduce synchronization,
  repository inventories or central managed-file enforcement.
- Add reusable tools for demonstrated needs. Keep independent products in their
  own repositories. Document actual inputs, outputs and checks; do not create
  speculative frameworks or empty resource directories.
