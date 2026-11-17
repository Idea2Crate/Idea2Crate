# Project instructions

This repository owns its instructions, hooks and CI. Maintain them here through
the project's normal review process. This template is a starting point; retain
only rules appropriate to the project and add its verified development commands
and links to product and architecture documents.

## Responsibility and scope

- A human is accountable for contributions made with an agent, including
  correctness, review responses, licensing and publication.
- Establish the intended outcome and observable acceptance criteria. Read the
  current code and project documents before choosing an implementation.
- Preserve unrelated work. Keep changes focused, remove superseded code and
  state, and prefer existing libraries and platform behavior over new machinery.
- Keep interfaces small and explicit. Separate reusable logic from platform
  adapters; explain compatibility changes and provide a migration when needed.
- Use dedicated patch/edit tools for text changes. Use a programmatic rewrite
  only when the transformation itself requires parsing or computation; explain
  why and inspect its diff.
- Apply only relevant skills after reading their instructions. A skill does not
  authorize actions beyond the user's request. Ask about material conflicts;
  continue independent work that does not depend on the answer.

## Checks and testing

- Use the project's pinned toolchain and dependency lockfiles. Run its formatter,
  lint and relevant tests, including documented required checks.
- Do not add unit tests for simple, reversible, low-impact changes, or tests that
  merely restate the implementation.
- Add unit tests only for important, error-prone behavior. Implementation
  complexity, file processing, configuration validation or network requests
  alone do not justify adding tests.
- Test observable behavior and failure boundaries. Focus on inputs and outputs;
  avoid checking internal fields one by one, implementation details or every
  conditional branch.
- Prefer existing checks, targeted smoke tests and verification with real inputs
  and outputs.
- Temporary tools, one-off scripts and experimental helpers usually need no new
  unit tests. Validate them by running them and fix issues found in actual use.
- When changing related logic, simplify or remove redundant, obsolete or brittle
  tests, including tests that repeat behavior handled by libraries or frameworks.
- Keep tests appropriate to the change. Once relevant checks pass, broaden or
  repeat testing only for new changes, failures or unresolved concerns.
- Review the final diff and report checks actually run, failures and material
  gaps. Never claim an unperformed acceptance check.

## Contributions and publication

- Follow the repository's branch and review process. Do not bypass required
  checks, hooks or branch protections. Preserve security and LFS hooks when
  installing the project's local hooks.
- Use Conventional Commits: `type(scope): description`, with optional scope and
  `!` for breaking changes. Explain incompatible changes and migration in the
  body. Keep PR titles short.
- PR bodies and PR progress comments contain only problem and fix lists for
  contributors without internal context. Omit conversations, debugging history
  and piles of implementation details. Put technical review findings in comments
  on the corresponding PR. Keep personal paths and internal coordination out
  of repository documentation.
- Ordinary changes and merges do not authorize a release, release-version bump,
  tag, package publication, distribution upload or paid service. Perform those
  only when explicitly requested. Keep publication credentials out of PR jobs.

## Secrets, data and dependencies

- Never commit credentials, private keys, tokens, personal data, private
  screenshots, local configuration, caches or build output. Do not print them
  in logs, PRs, artifacts or error reports. Use protected secret storage with
  minimal scope. If a secret is exposed, stop propagation and report it for
  revocation; deleting a file does not revoke a credential.
- Check dependency licenses and preserve required copyright and attribution.
  Pin external workflow actions to full commit SHAs; use minimal permissions
  and finite timeouts. Do not expose secrets to untrusted PR execution.
- Respect user data and system permissions. Tests use generated or disposable
  data and isolated destinations. Desktop interaction, real screenshots,
  clipboard writes, login items and installation require task authorization;
  simulated UI tests do not prove native platform behavior.
