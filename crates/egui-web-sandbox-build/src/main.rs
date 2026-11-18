use egui_web_sandbox_build::{Builder, Result};
use std::{env, path::Path};

const USAGE: &str = "Usage: egui-web-sandbox [path/to/Cargo.toml]";

fn main() {
    if let Err(error) = run() {
        eprintln!("egui-web-sandbox: {error}");
        std::process::exit(1);
    }
}

fn run() -> Result<()> {
    let mut args = env::args_os().skip(1);
    let manifest = args.next().unwrap_or_else(|| "Cargo.toml".into());
    if args.next().is_some() {
        return Err(USAGE.into());
    }
    if manifest == "--help" || manifest == "-h" {
        println!("{USAGE}\nBuild the package's WASM library and serve it on localhost.");
        return Ok(());
    }
    if Path::new(&manifest)
        .file_name()
        .is_none_or(|name| name != "Cargo.toml")
    {
        return Err(USAGE.into());
    }
    Builder::default().serve(Path::new(&manifest))
}
