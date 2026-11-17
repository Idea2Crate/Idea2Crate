# Idea2Crate Standards

One reviewed source for shared agent instructions, skills and pre-commit checks.
Idea2Crate turns ideas and real needs into working, verifiable software with
agents. Projects have independent goals and designs; replicating an existing
product is outside our scope.

Edit common rules here. Each enrolled repository receives an identical root
`AGENTS.md`, shared skills, a generated pre-commit configuration and a separate
source lockfile. Format version 2 uses `.agents/skills/idea2crate-*` for shared skills and
`.idea2crate/LICENSE` for their source notice; other skill names remain owned by
the project. Project-specific requirements live in `docs/project.md` and the
documents it links to.

After a central merge, automation opens or updates one sync PR per repository.
Each central PR first needs human approval of its final checked diff, in chat or
GitHub. Any later edits require renewed approval.
An independent GitHub App check validates the exact payload, then enables
auto-merge subject to all required checks. Normal contributors cannot merge
unapproved changes to managed copies. Routine syncs need maintainer attention
for conflicts and failures; explicit enrollment and format migration are reviewed
separately.

Approved source revisions using the current format remain valid during rollout. A reviewed
`revoked_revisions` entry withdraws a specific source version, with a reason;
the next reconciliation rejects it while allowing an exact upgrade to repair
the installed version. Neither synchronization nor revocation is atomic across
repositories.

Only the current bundle format is supported. The two existing repositories are
migrated explicitly; runtime code does not reconstruct previous formats. See
[enrollment](docs/enrollment.md) for the reviewed migration procedure.

```sh
python3 -m unittest discover -s tests -v
python3 -m standards plan --repository Idea2Crate/Snip2Pin
```

Read [development](docs/project.md), [setup and trust model](docs/automation.md)
and [enrollment](docs/enrollment.md). Python 3.9+ and Git are required. There are
no third-party runtime Python dependencies. No package publication is required.

The default checker uses central GitHub Actions on a ten-minute schedule and
after central merges. GitHub may delay scheduled runs; this is not an instant
webhook service. Required checks remain pending until validation runs, and no
target PR can grant itself a successful check. A maintainer can run the central
workflow manually when an immediate result is needed.
