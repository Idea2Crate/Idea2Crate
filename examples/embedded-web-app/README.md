# Distribute an egui App as a standalone server

```sh
cargo run --locked -p egui-embedded-web-app
cargo run --locked -p egui-embedded-web-app -- 8000 --bind 127.0.0.1
```

The example uses the same App and Web entry as [web-app](../web-app/).
Its [Cargo manifest](Cargo.toml) adds native server and build dependencies.
`build.rs` compiles the WASM library and generates its Web resources.
`src/main.rs` embeds those resources and starts the server with `serve_from_args`.
The executable accepts a positional port or `--port PORT`, plus
`--bind ADDRESS` (`-b`).

The resulting executable prints a loopback URL and serves the app from an empty
directory without Cargo or source files. Stop the server with Ctrl+C.

See the [build API](../../crates/egui-web-sandbox-build/) for configuring Cargo
features when embedding. Use `server::serve(include_app!())` to start with defaults
without reading the process's arguments, or `server::Server::bind` to control
the port and run on a thread managed by the application.
