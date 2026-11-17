# egui-web-sandbox-build

Build-script API for embedding an egui Web application in a native executable.
Paired with the `server` feature of `egui-web-sandbox`.

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
The package's normal source tracking triggers rebuilds. For inputs outside the
package, use build.rs rerun declarations that cover both package and external
inputs. `Builder` configures default/Web-only Cargo features when needed.

No compiler, source tree or binding generator is required at runtime. This
build-only dependency does not become part of the executable. It currently uses
wasm-bindgen 0.2.130; mismatched artifacts are rejected by wasm-bindgen.

See [quick start](../egui-web-sandbox/docs/quick-start.md) and the complete
[example](../../examples/web-app/). Not published to crates.io.
