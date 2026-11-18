mod cli;
mod server;

use cli::{LISTEN_HELP, Options};
use std::{collections::BTreeMap, error::Error, path::Path};
use wasm_bindgen_cli_support::Bindgen;

type Result<T> = std::result::Result<T, Box<dyn Error + Send + Sync>>;

const USAGE: &str = "Usage: egui-web-sandbox <app.wasm> [PORT] [OPTIONS]";

fn main() {
    if let Err(error) = run() {
        eprintln!("egui-web-sandbox: {error}");
        std::process::exit(1);
    }
}

fn run() -> Result<()> {
    let options = Options::parse()?;
    if options.help {
        println!(
            "{USAGE}\n\nServe an app compiled for wasm32-unknown-unknown with wasm-bindgen.\n\n{LISTEN_HELP}"
        );
        return Ok(());
    }
    let wasm = options.wasm.ok_or(USAGE)?;
    server::serve(generate_assets(&wasm)?, options.address)
}

fn generate_assets(wasm: &Path) -> Result<Vec<(String, Vec<u8>)>> {
    let mut output = Bindgen::new()
        .input_path(wasm)
        .web(true)?
        .omit_default_module_path(false)
        .typescript(false)
        .out_name("app")
        .generate_output()?;
    let mut assets = vec![
        ("app.js".into(), output.js().as_bytes().to_vec()),
        ("app_bg.wasm".into(), output.wasm_mut().emit_wasm()),
    ];
    if let Some(start) = output.start() {
        assets.push(("app_bg.js".into(), start.as_bytes().to_vec()));
    }

    // Match wasm-bindgen's output layout so its generated imports still resolve.
    for (crate_id, snippets) in output.snippets() {
        for (index, source) in snippets.iter().enumerate() {
            assets.push((
                format!("snippets/{crate_id}/inline{index}.js"),
                source.as_bytes().to_vec(),
            ));
        }
    }
    for (path, source) in output.local_modules() {
        assets.push((format!("snippets/{path}"), source.as_bytes().to_vec()));
    }
    if !output.npm_dependencies().is_empty() {
        let dependencies: BTreeMap<_, _> = output
            .npm_dependencies()
            .iter()
            .map(|(name, (_, version))| (name, version))
            .collect();
        let manifest = serde_json::json!({ "dependencies": dependencies });
        assets.push(("package.json".into(), serde_json::to_vec_pretty(&manifest)?));
    }
    Ok(assets)
}
