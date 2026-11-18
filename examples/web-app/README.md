# Run an egui App with the sandbox command

Run from the repository root:

```sh
cargo install --locked --bin wasm-bindgen wasm-bindgen-cli --version '=0.2.130'
cargo build --locked -p egui-web-app --lib --target wasm32-unknown-unknown
wasm-bindgen --target web --no-typescript --out-dir target/web/notes target/wasm32-unknown-unknown/debug/egui_web_app.wasm
cargo build --locked -p egui-web-sandbox --features cli --bin egui-web-sandbox
./target/debug/egui-web-sandbox pack target/web/notes/egui_web_app.js target/web/notes.egsx --app-version 0.1.0 --wasm-bindgen-version 0.2.130
./target/debug/egui-web-sandbox target/web/notes.egsx
```

Open the loopback URL printed by the command. Copy only `notes.egsx` and
the CLI to another machine to serve the same build.

- `src/app.rs` is an ordinary eframe App with UI and logic hooks.
- `src/lib.rs` returns an eframe `AppCreator`. The sandbox attribute exports
  the browser's `start()` entry.
- `Cargo.toml` declares the WASM library and its dependencies.

Build-time prerequisites are Rust, the `wasm32-unknown-unknown` target and the
matching `wasm-bindgen` tool. The server reads the bundle in memory. Native
services remain the project's responsibility.

See the [quick start](../../crates/egui-web-sandbox/docs/quick-start.md) for using
the command from another project.
