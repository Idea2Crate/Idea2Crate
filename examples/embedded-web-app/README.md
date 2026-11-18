# Distribute an egui App as a standalone server

```sh
cargo run --locked -p egui-embedded-web-app
```

The example uses the same App and Web entry as [web-app](../web-app/).
Its [Cargo manifest](Cargo.toml) adds native server and build dependencies.
`build.rs` compiles the WASM library and generates its Web resources.
`src/main.rs` embeds those resources and starts the server.

The resulting executable prints a loopback URL and serves the app from an empty
directory without Cargo or source files. Stop the server with Ctrl+C.

See the [build API](../../crates/egui-web-sandbox-build/) for configuring Cargo
features when embedding, and `server::Server::bind(include_app!(), port)` for
controlling the port or running the server on a thread managed by the application.
