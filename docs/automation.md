# Setup and trust model

## Central approval and installation

1. Protect central `main`: require PRs, successful `verify`, resolved discussions
   and no force pushes or deletions. A human maintainer explicitly approves each
   central PR's final diff after checks pass, in chat or GitHub. The agent may
   then merge that exact head; later edits invalidate approval. Asking to draft
   or update a rule is not approval to merge its eventual implementation.
   The current solo-maintainer ruleset requires zero GitHub approval reviews so
   chat approval is possible. This is an authorization procedure, not technical
   proof of human review. Limit merge rights to authorized maintainers; the
   automation never opens or merges central rule PRs.
2. Register a private organization-owned GitHub App, using the permissions in
   `.github/app-manifest.json`: repository contents, PRs and checks write;
   metadata read. Disable webhooks for this scheduled implementation. It needs
   no organization administration, workflow write or user permissions.
3. Install it only on `standards` and enrolled repositories. This grants write
   access to those repositories and must be explicitly approved by the owner.
4. In central Actions variables set `STANDARDS_APP_ID` to the numeric App ID,
   `STANDARDS_BOT_ID` to the numeric user ID of the App's bot. Store the private
   key only as central Actions secret `STANDARDS_APP_PRIVATE_KEY`. Do not put it
   in a local config, source, log, issue or artifact. Generate/transfer credentials
   through GitHub's protected settings; documentation is not permission to expose
   them. Each reconciliation job mints a short-lived installation token for
   exactly one enrolled downstream repository and revokes it on completion.
   Runtime tokens exclude the central repository even if the App is installed
   there. Never omit an empty repository scope and accidentally grant all repos.
5. Complete the first repository migration as described in [enrollment.md](enrollment.md).
6. Run the central reconcile workflow once. On each target, add required check
   `Idea2Crate / standards`, **bound to this App's numeric integration ID**, to
   its existing protected-main ruleset. Preserve every existing required check.
   The check must have been submitted once for GitHub to offer its producer.
7. Require PRs but zero repeated human reviews for exact syncs; enable repository
   auto-merge and squash merging. Auto-merge cannot override a required human
   approval, a failed CI check or unresolved review. No App gets a ruleset bypass.

Run a tamper PR and verify it cannot merge before describing enforcement as
active. Installing code alone does not configure branch protection or an App.

## Automatic operation

Central `push main` and manual runs synchronize the latest merged source.
Scheduled runs check drift, retry sync and validate open PRs every ten minutes.
GitHub scheduling is best-effort and can be delayed or disabled on inactive public
repositories. Monitor the central Actions workflow and restore it if disabled.
There is no hosted server, public webhook endpoint or downstream secret. If
instant checks become necessary, the same checker can be called by a separately
deployed App webhook handler; that deployment is outside this implementation.

A credential-free planning job lists enrolled targets. Independent matrix jobs
mint their own repository-scoped tokens with `fail-fast: false`; one repository's
missing App installation or API failure does not stop another repository's job.
An empty registry runs no credentialed jobs. A malformed central registry fails
planning and needs correction in the central repository.

Each run checks out central main, including its history, with credentials
disabled in Git. It mints a scoped App token only after checkout. PRs against the
central repository run tests only and never receive the App key.

For each downstream PR, the checker:

- Reads Git trees and blobs through the API; never checks out or executes target
  code. Truncated trees and unrecognized lockfiles fail closed.
- Verifies installed files against an actual ancestor commit of central main.
  The lockfile's self-reported hash alone proves nothing. Intact older versions
  remain valid during rollout, avoiding a freeze of ordinary development.
- Rejects extra/nested `AGENTS.md`, `AGENTS.override.md`, managed-path aliases,
  symlinks in managed paths, unapproved central skills and modified generated
  pre-commit configuration. The current format reserves `idea2crate-*` skill
  names; other project skill names are allowed.
- Allows routine changes to managed files only in a same-repository PR opened by the
  configured App bot on `codex/sync-standards`, with the exact latest payload
  and no unrelated changes. App IDs, branch names, labels or checksums supplied
  by PR code cannot override central enrollment or the expected file contents.
- Enrollment or deliberate format migration may use a separately human-approved
  exact `bootstrap_head`/`bootstrap_base` grant. The new payload must still match
  an active approved source revision in the current format; no old format is
  interpreted. A different head or baseline cannot reuse that approval.
- Rechecks the full PR identity (head, target repository/branch, author, draft
  and open state) and current enrolled base before publishing a result and again
  before enabling auto-merge. Checks are tied to a head SHA; every later commit
  needs its own successful result. GitHub's auto-merge API cannot atomically bind
  the mutation to this entire snapshot; keep strict required checks enabled and
  restrict who may edit the bot's PRs and branch. A later metadata change needs
  a fresh reconciliation; these checks are not a cross-API transaction.
- Enables auto-merge only for a verified sync PR. Normal application PRs are
  checked but are not automatically merged by this automation.

Sync copies exact bytes, records the source revision separately and deletes
retired central files while preserving project skills. `.idea2crate/LICENSE`
contains the central copyright/permission notice; the project license is not
replaced. Only the current format, version 2, is supported. A previous-format
installation requires the explicit migration in [enrollment.md](enrollment.md),
not a compatibility branch in the runtime. Existing sync PRs are updated using fast-forward-only commits; a
concurrent change is not force-overwritten. The sync commit's baseline parent
must be an ancestor of the current enrolled branch, and its changes from that
baseline must be limited to exact central file paths. Merging an unrelated
feature branch into the sync branch stops automation instead of discarding it.
Identical pending payloads are reused. Different repositories converge
independently; this is not an atomic cross-repository transaction.

An ordinary contribution rejected by policy receives a failed App check; this is
an expected checker outcome and does not itself fail the central job. Failure
to validate the App's own PR, a changing PR/base, an API error or incomplete
verification fails the repository's central job. This distinguishes an invalid
contribution from broken synchronization. Other targets and PRs are still tried.

## Withdrawing a source revision

Add a full central source SHA and a public, nonempty reason to
`revoked_revisions` in `repositories.json`, through the same final human approval
as other central changes. Empty `{}` means no versions are withdrawn. Do not
insert credentials or private incident details into the reason.

The current central checker rejects PRs using withdrawn revisions. Ordinary
approved older source revisions using the current format remain valid until
synchronized. Withdrawal does not rewrite history or prevent the checker from
reconstructing withdrawn content in that format: the bot may upgrade an exact
withdrawn baseline or sync branch to an active source revision. This exception
repairs known content; it cannot bless arbitrary edits or interpret older formats.
Local offline hooks cannot discover later withdrawals from their pinned source.
The main-push workflow and subsequent reconciliations propagate withdrawals;
already-published checks are not instantaneously invalidated across repositories.

## Boundaries and recovery

The guarantee is that unauthorized managed-file changes cannot merge to a
properly protected upstream branch. People can still edit local files or forks;
repository administrators can change protection settings. Instructions cannot
prove that an agent follows them. Keep App credentials and central approval
privileges limited to maintainers.

`pre-commit` verifies against its immutable source checkout, without network
requests during the integrity hook. Local hooks can be skipped; the independent
App check is the authoritative gate. The credential-pattern hook is a useful
baseline, not complete secret detection; enable GitHub secret scanning and push
protection separately when available.

When a run fails, fix the cause and rerun centrally. API failures do not bless
unknown content. Do not remove required checks to unblock a sync. Rollbacks are
new reviewed central commits restoring the desired content, then normal syncs.
They do not require rewriting history or editing downstream copies.

For an interrupted local `apply`, inspect its diff before recovery. Restore only
the affected central files from the target's known, exact installed baseline,
preserving project work, then rerun `apply` from the merged central revision.
Restoring known bytes is recovery, not editing the policy locally. Never reset
the entire worktree or edit a lock checksum to accept partial content. New files
can be removed only after checking that the interrupted operation created them.
