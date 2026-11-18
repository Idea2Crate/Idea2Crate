mod bundle;
mod cli;
mod server;

use cli::{LISTEN_HELP, Options, PackOptions};
use std::{env, error::Error};

type Result<T> = std::result::Result<T, Box<dyn Error + Send + Sync>>;

const USAGE: &str = "Usage: egui-web-sandbox <app.egui-sandbox> [PORT] [OPTIONS]";
const PACK_USAGE: &str = "Usage: egui-web-sandbox pack <app.js> <app.egui-sandbox> [--app-version VERSION] [--wasm-bindgen-version VERSION]";

fn main() {
    if let Err(error) = run() {
        eprintln!("egui-web-sandbox: {error}");
        std::process::exit(1);
    }
}

fn run() -> Result<()> {
    let mut args = env::args_os().skip(1).peekable();
    if args.next_if_eq("pack").is_some() {
        if args
            .peek()
            .is_some_and(|arg| arg == "--help" || arg == "-h")
        {
            println!("{PACK_USAGE}\n\nPackage the output of wasm-bindgen --target web.");
            return Ok(());
        }
        return bundle::pack(PackOptions::parse(args)?);
    }
    let options = Options::parse(args)?;
    if options.help {
        println!("{USAGE}\n\nServe a prebuilt web app bundle.\n\n{LISTEN_HELP}\n\n{PACK_USAGE}");
        return Ok(());
    }
    let path = options.bundle.ok_or(USAGE)?;
    let bundle = bundle::open(&path)
        .map_err(|error| format!("could not open {}: {error}", path.display()))?;
    server::serve(&bundle.entry, bundle.assets, options.address)
}
