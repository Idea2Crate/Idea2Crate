use egui_web_sandbox::server::cli::{LISTEN_HELP, Options};
use egui_web_sandbox_build::{Builder, Result};

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
            "Usage: egui-web-sandbox [path/to/Cargo.toml] [PORT] [OPTIONS]\n\nBuild and serve the package's WASM library.\n\n{LISTEN_HELP}"
        );
        return Ok(());
    }
    let manifest = options.manifest.unwrap_or_else(|| "Cargo.toml".into());
    Builder::default().serve_at(&manifest, options.address)
}
