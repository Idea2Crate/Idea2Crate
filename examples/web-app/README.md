# Run an egui App with the sandbox command

Run from the repository root:

```sh
cargo run --locked -p egui-web-sandbox-build --features cli --bin egui-web-sandbox -- examples/web-app/Cargo.toml
```

Open the loopback URL printed by the command. The command supplies the server
and HTML and compiles this package's library for `wasm32-unknown-unknown`.

- `src/app.rs` is an ordinary eframe App with UI and logic hooks.
- `src/lib.rs` supplies its constructor to the Web launcher.
- `Cargo.toml` declares the WASM library and its dependencies.

Build-time prerequisites are Rust, the wasm32-unknown-unknown target and the
project's locked dependencies. Native services remain the project's
responsibility; this example uses only ordinary egui controls.

See the [quick start](../../crates/egui-web-sandbox/docs/quick-start.md) for using
the command from another project, or the [embedded example](../embedded-web-app/)
for building a self-contained native executable.
