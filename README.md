# Idea2Crate

Reusable tools, skills and templates for turning ideas into working, verifiable
software with agents. Independent products live in their own repositories.
Projects have their own goals and designs; replicating an existing product is
outside our scope. Reuse compatible open-source components with attribution.

Each repository maintains its own `AGENTS.md`, hooks and CI. Use these resources
when needed; this repository does not synchronize or enforce project rules.

## Available resources

| Resource | Purpose |
| --- | --- |
| [egui-web-sandbox](crates/egui-web-sandbox/) | Launch an egui App in a browser, observe controls and test real input in isolated instances |
| [egui-web-sandbox-build](crates/egui-web-sandbox-build/) | Automatically build and embed a Web app into a self-contained server executable |
| [Complete Web server example](examples/web-app/) | Run the executable and open its URL; no external HTTP server or template copying |
| [idea2crate-repo-setup](skills/idea2crate-repo-setup/SKILL.md) | Create a project or update its development rules when explicitly invoked |

The Rust crate is available as a Git dependency pinned to a reviewed commit;
it is not published to crates.io. Start with its [quick start](crates/egui-web-sandbox/docs/quick-start.md).

## Repository setup skill

[`idea2crate-repo-setup`](skills/idea2crate-repo-setup/SKILL.md) creates a new
project's development setup or updates an existing project's instructions and
checks. Invoke it explicitly; it is not part of ordinary application changes.
It adapts a baseline to the project and preserves existing project requirements.

Examples:

```text
Use $idea2crate-repo-setup to prepare a new Rust library from this idea:
<purpose and intended users>. Create local files only.

Use $idea2crate-repo-setup to update this repository's contribution rules.
Keep its product requirements and existing checks; show the resulting diff.
```

## Project-local installation

Ask your agent's skill installer to install from:

- Repository: `Idea2Crate/Idea2Crate`.
- Path: `skills/idea2crate-repo-setup`.
- Revision: a full reviewed commit SHA.
- Destination: the target repository's `.agents/skills/` directory.

For Codex, use the bundled `skill-installer` with those inputs. The helper accepts
`--repo`, `--path`, `--ref` and `--dest`; have the agent locate its bundled script
rather than putting a machine-specific script path into project instructions.
Downloading the skill does not require a persistent clone of this repository.
The installer may download a repository archive to extract the selected path.

Add this rule to the target project's `.gitignore`:

```gitignore
/.agents/skills/idea2crate-*/
```

Installed skills are local dependencies, not project contributions. Preserve
their bundled `LICENSE`. Do not overwrite an existing installation or local
edits without inspection. Updating an installed skill does not itself update
project rules: a human must invoke it for that task. Project-owned skills with
other names can remain tracked normally. New clones and worktrees need their own
local installation.

This skill needs no custom installer, package publication, organization Secret,
GitHub App, consumer registry or synchronization workflow. Projects decide when
to use it and own the resulting files without a runtime dependency here.

## Adding reusable tools

Add a tool or template when an actual task needs it. Document its purpose,
inputs, outputs, dependencies and verification alongside it. Keep standalone
products in separate repositories rather than growing this into a shared
application runtime. Skills may bundle supporting scripts when they are useful
to that workflow.

See [development](docs/project.md) for this repository's own checks.
