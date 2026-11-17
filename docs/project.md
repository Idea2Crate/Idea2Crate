# Standards development

This repository is the canonical source of Idea2Crate's shared `AGENTS.md`,
skills and pre-commit profiles. It contains infrastructure, not application code.
Python 3.9+ and Git are required; runtime code has no third-party dependencies.

- `AGENTS.md`: the complete common instructions, identical across projects.
- `.agents/skills/idea2crate-*`: shared skills, copied into the same discoverable
  location. Other skill names are project-owned.
- `profiles.json`: common pre-commit checks and explicit project profiles.
- `repositories.json`: reviewed enrollment, target branches and source revocations.
- `standards/`: deterministic bundle generation, GitHub API client and automation.
- `hooks/`: local pre-commit entry points.
- `tests/`: isolated regression tests; no real credentials or GitHub writes.

Run `python3 -m unittest discover -s tests -v`. Runtime code supports Python 3.9+;
development tooling needs Python 3.11+ for pre-commit 4.7.0. Install that pinned tool and run
`pre-commit install`, then `pre-commit run --all-files` before contributing.
The secret hook detects recognizable token and private-key patterns; it is not
a complete secret scanner. GitHub secret scanning/push protection adds coverage.
Python lint and formatting use the pinned Ruff pre-commit hooks. If another tool
owns `core.hooksPath`, use its supported pre-commit integration; do not replace
or disable the existing security or LFS hooks.

Use a short `codex/` branch and a PR. A human must explicitly approve each central
PR's final checked diff in chat or GitHub. Any later edits require approval again;
the agent may merge the approved commit after confirming its SHA and checks.
Exact downstream copies need no repeated approval and may merge automatically.
Read [automation.md](automation.md) before changing sync or enforcement.
