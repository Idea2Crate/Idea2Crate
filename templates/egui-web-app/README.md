# egui Web entry template

This is a complete browser entry for an ordinary `eframe::App`.

1. Copy this directory into your project. Replace the sandbox's repository-local
   `path` dependency with `git = "https://github.com/Idea2Crate/Idea2Crate"` and
   `rev = "REVIEWED_COMMIT_SHA"`; keep `version = "0.1.0"` and `features = ["launcher"]`.
2. Add your application crate as a WASM dependency. Replace the factory in
   `src/lib.rs` with its constructor; `src/app.rs` is only the sample application.
   Your application owns native-service adapters and generated test data.
3. Run `rustup show`, then `python3 build.py`. If wasm-bindgen is missing or has
   the wrong version, the builder prints the matching installation command.
4. Serve the result: `python3 -m http.server 8765 --bind 127.0.0.1 --directory .build/site`.

Open <http://127.0.0.1:8765/>. Commit the generated Cargo.lock and use
`python3 build.py --locked` in CI. The builder obtains `launch.mjs` and
`client.mjs` from the actual Cargo-resolved sandbox, so no sibling checkout or
separate JS download is needed. Configure `--out-dir` when a build system needs
a different destination. Generated site files stay outside version control.

`index.html` handles loading, startup errors and page cleanup. For mounting in
an existing page, call `mount()` from the generated `launch.mjs`; it returns an
idempotent stop function. The returned Rust WebApp is retained until unmount.

The sample Notes app uses normal `App::logic` and `App::ui`; business code has
no sandbox imports. Preserve this template's license notice when copying it.
