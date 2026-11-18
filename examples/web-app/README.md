# Run an egui App with the sandbox command

Run from the repository root:

```sh
cargo build --locked -p egui-web-app --lib --target wasm32-unknown-unknown
cargo build --locked -p egui-web-sandbox-build --features cli --bin egui-web-sandbox
./target/debug/egui-web-sandbox target/wasm32-unknown-unknown/debug/egui_web_app.wasm
```

Open the loopback URL printed by the command. It takes the compiled WASM file,
generates its browser bindings and serves the app with the supplied HTML.

- `src/app.rs` is an ordinary eframe App with UI and logic hooks.
- `src/lib.rs` supplies its constructor to the Web launcher.
- `Cargo.toml` declares the WASM library and its dependencies.

Build-time prerequisites are Rust, the wasm32-unknown-unknown target and the
project's locked dependencies. Native services remain the project's
responsibility; this example uses only ordinary egui controls.

See the [quick start](../../crates/egui-web-sandbox/docs/quick-start.md) for using
the command from another project, or the [embedded example](../embedded-web-app/)
for building a self-contained native executable.
