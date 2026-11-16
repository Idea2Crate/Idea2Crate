# Setup and trust model

## Central approval and installation

1. Protect central `main`: require PRs, successful `verify`, resolved discussions
   and no force pushes or deletions. A human maintainer reviews and merges rule
   changes. Solo maintenance does not require a second person.
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
   them. The token action mints a short-lived installation token and revokes it.
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
  symlinks, extra shared skills and modified generated pre-commit configuration.
- Allows changes to managed files only in a same-repository PR opened by the
  configured App bot on `codex/sync-standards`, with the exact latest payload
  and no unrelated changes. App IDs, branch names, labels or checksums supplied
  by PR code cannot override central enrollment or the expected file contents.
- Rechecks head and base before publishing a result. Checks are tied to a head
  SHA; every later commit needs its own successful result. Keep strict required
  checks enabled so a changed base cannot reuse stale validation.
- Enables auto-merge only for a verified sync PR. Normal application PRs are
  checked but are not automatically merged by this automation.

Sync copies exact bytes, records the source revision separately and deletes
retired managed files. Existing sync PRs are updated using fast-forward-only
commits; a concurrent change is not force-overwritten. Mixed-content bot branches
stop for review. Identical pending payloads are reused. Different repositories
converge independently; this is not an atomic cross-repository transaction.

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
