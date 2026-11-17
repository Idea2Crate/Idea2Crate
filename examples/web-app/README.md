# Self-contained egui Web server

Run from the repository root:

```sh
cargo run --locked -p egui-web-app
```

Open the loopback URL printed by the executable. No HTML setup, external HTTP
server, Python or wasm-bindgen CLI is needed.

- `src/app.rs` is an ordinary eframe App with UI and logic hooks.
- `src/lib.rs` supplies its constructor to the Web launcher.
- `build.rs` builds the WASM library and embeds its generated bindings.
- `src/main.rs` serves the embedded app through the sandbox's Rust API.

The finished executable works from an empty directory without Cargo or source
files. It chooses an unused loopback port; separate processes can run in parallel.
Each browser page has its own App instance. Stop the server with Ctrl+C.

Build-time prerequisites are Rust, the wasm32-unknown-unknown target and the
project's locked dependencies. Native services remain the project's
responsibility; this example uses only ordinary egui controls.

See the [quick start](../../crates/egui-web-sandbox/docs/quick-start.md) for adding
the APIs to an existing project. There is no directory-copy installation step.
