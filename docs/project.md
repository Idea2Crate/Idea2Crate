# Shared tools and skills development

Canonical repository: <https://github.com/Idea2Crate/Idea2Crate>.
This repository contains reusable tools, skills and templates. It is not a
central policy service. Repository setup skills are explicitly invoked.

- `skills/idea2crate-repo-setup/`: the entry point, invocation metadata, baseline
  asset and license needed to create or update a project's development setup.
- `crates/egui-web-sandbox/`: reusable egui observations, browser clipboard/reset
  environment, optional Web launcher, browser clients and regression tests.
- `crates/egui-web-sandbox-build/`: build-script API that compiles the current
  package's WASM library and emits its embedded resources into Cargo OUT_DIR.
- `examples/web-app/`: a self-contained server executable with an ordinary App
  factory, automatic Web build and the native server API.
- `Cargo.toml`, `Cargo.lock`, `rust-toolchain.toml`: the Rust workspace and its
  reproducible development toolchain; this workspace has no application dependency.
- `AGENTS.md`: this repository's own instructions, independently maintained.
- `.pre-commit-config.yaml`: this repository's checks, not an exported profile.
- `hooks/check_secrets.py`: a local credential-pattern guard. It complements
  GitHub secret scanning and push protection, not a complete secret scanner.
- `tests/test_secrets.py`: the existing observable credential rejection check.

Keep installed skills self-contained: their references and assets must live
inside the skill directory. Do not introduce a dependency on a sibling checkout,
consumer list or privileged service. The setup skill uses a template as advice,
then adapts the result to the target repository. It must not silently reconfigure
branch protections or change the user's existing hooks.

Use the pinned Rust toolchain, Node.js 24, Python 3.11+ and pre-commit 4.7.0 for
development. Install the local hooks with
`pre-commit install`, then run `pre-commit run --all-files`. If another tool owns
`core.hooksPath`, use its supported integration; preserve security and LFS hooks.
CI runs these checks in one Ubuntu `verify` job, which is required for merging.
The credential guard itself uses Python 3.9+ and the standard library.

Rust hooks run formatting, WASM Clippy and native server/build-helper Clippy.
CI builds the self-contained executable and browser fixtures, then runs browser
interaction, lifecycle and empty-PATH/cwd server regressions. The launcher passes App directly to eframe;
do not replace it with a proxy that silently drops application lifecycle hooks.
Build and run commands live in the crate's [quick start](../crates/egui-web-sandbox/docs/quick-start.md).
Keep the browser library independent of consumers, including their manifests,
native services and build commands. Dependents select their own compatible egui
source. Do not rely on a consuming workspace to enable required Cargo features.

For a new or changed skill, also run your agent's bundled skill validator and
check its behavior against realistic requests: creating a new project, updating
an existing one without losing rules, and ordinary coding that must not invoke
the setup skill. Structural validation alone does not prove correct behavior.
Documentation and template edits do not need new unit tests.

Use a short `codex/` branch and a PR. A human explicitly approves the final
checked diff of each PR here, in chat or GitHub. Later edits require approval
again. Do not publish a package or create a release unless explicitly requested.
