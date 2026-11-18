# egui-web-sandbox-build

The `cli` feature provides the `egui-web-sandbox <app.wasm>` command. Pass the
WASM library already built by your project. The command generates browser bindings
and prints a loopback URL. See the [quick start](../egui-web-sandbox/docs/quick-start.md).

To build your own standalone executable instead, pair the build-script API with
the `server` feature of `egui-web-sandbox`.

```rust,ignore
fn main() {
    egui_web_sandbox_build::build().expect("build embedded Web app");
}
```

The current package must have a cdylib library exporting the launcher's async
`start()`. The helper compiles that library for wasm32-unknown-unknown using
Cargo.lock, generates JS bindings, and writes an embed expression to OUT_DIR.
`egui_web_sandbox::include_app!()` includes that expression in the native binary.
WASM child builds skip bundling to avoid recursion.

A separate target directory under OUT_DIR prevents nested Cargo lock contention.
Cargo's depfile tracks transitive local sources, included assets and manifests
so shared-library edits refresh the embedded app too. Project-specific code
generators still declare their own extra inputs in build.rs. `Builder`
configures default/Web-only Cargo features when needed.

No compiler, source tree or binding generator is required at runtime. This
build-only dependency does not become part of the executable. It currently uses
wasm-bindgen 0.2.130; mismatched artifacts are rejected by wasm-bindgen.

See [quick start](../egui-web-sandbox/docs/quick-start.md) and the complete
[embedded example](../../examples/embedded-web-app/). Not published to crates.io.
