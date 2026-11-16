# Standards development

This repository is the canonical source of Idea2Crate's shared `AGENTS.md`,
skills and pre-commit profiles. It contains infrastructure, not application code.
Python 3.9+ and Git are required; runtime code has no third-party dependencies.

- `AGENTS.md`: the complete common instructions, identical across projects.
- `.agents/skills/`: shared skills, copied into the same discoverable location.
- `profiles.json`: common pre-commit checks and explicit project profiles.
- `repositories.json`: the reviewed enrollment list and target default branches.
- `standards/`: deterministic bundle generation, GitHub API client and automation.
- `hooks/`: local pre-commit entry points.
- `tests/`: isolated regression tests; no real credentials or GitHub writes.

Run `python3 -m unittest discover -s tests -v`. Install pre-commit 4.7.0 and run
`pre-commit install`, then `pre-commit run --all-files` before contributing.
The secret hook detects recognizable token and private-key patterns; it is not
a complete secret scanner. GitHub secret scanning/push protection adds coverage.
Python lint and formatting use the pinned Ruff pre-commit hooks. If another tool
owns `core.hooksPath`, use its supported pre-commit integration; do not replace
or disable the existing security or LFS hooks.

Use a short `codex/` branch and a PR. Human approval belongs at the central rule
change; exact downstream copies may merge automatically after required checks.
Read [automation.md](automation.md) before changing sync or enforcement.
