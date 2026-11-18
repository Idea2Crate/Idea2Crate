---
name: idea2crate-egui-web-sandbox
description: Integrate or update egui-web-sandbox for browser demos and agent-driven UI tests of existing egui apps.
---

Read the consuming project's instructions and identify its existing eframe
factory, native entry point, web entry point, and platform adapters. Read
`crates/egui-web-sandbox/docs/quick-start.md` in
[Idea2Crate/Idea2Crate](https://github.com/Idea2Crate/Idea2Crate) at the revision
selected by the project.

## Share the application entry

- Use one ordinary eframe `AppCreator` and one initialization flow for both
  native and browser hosts. Keep the constructor, shared UI, and business logic
  free of WASM or Sandbox checks, including `cfg` branches and runtime probes.
  Let the host select platform capabilities. Keep target-specific imports,
  dependencies, and implementations in the host entry or platform adapters.
- Reuse the application's existing adapter for files, capture, and other
  platform operations. Build the WASM application and test its real interactions
  through an isolated browser context with disposable data.

## Sandbox compatibility exceptions

If a verified Sandbox incompatibility requires a separate behavior, obtain
explicit human approval before implementing the exception. Explain the exact
operation, the compatibility gap, and the smallest proposed workaround, then
wait for approval of that specific change.

Keep approved special cases in the host or adapter. Record the behavior, reason,
affected path, and removal condition in the consuming project's documentation.
When Sandbox supports the operation, migrate to the shared path, verify it in
the affected hosts, and remove the workaround and its active exception entry.
