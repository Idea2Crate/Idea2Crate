use egui_web_sandbox::server::cli::{LISTEN_HELP, Options};
use egui_web_sandbox_build::{Result, serve_wasm};

const USAGE: &str = "Usage: egui-web-sandbox <app.wasm> [PORT] [OPTIONS]";

fn main() {
    if let Err(error) = run() {
        eprintln!("egui-web-sandbox: {error}");
        std::process::exit(1);
    }
}

fn run() -> Result<()> {
    let options = Options::parse(true)?;
    if options.help {
        println!(
            "{USAGE}\n\nServe an app compiled for wasm32-unknown-unknown with wasm-bindgen.\n\n{LISTEN_HELP}"
        );
        return Ok(());
    }
    let wasm = options.wasm.ok_or(USAGE)?;
    serve_wasm(&wasm, options.address)
}
