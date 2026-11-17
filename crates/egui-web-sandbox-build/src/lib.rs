//! Build-script integration for a self-contained egui Web server executable.
use std::{
    env,
    error::Error,
    fs,
    io::{BufRead, BufReader},
    path::{Path, PathBuf},
    process::{Command, Stdio},
};

pub type Result<T> = std::result::Result<T, Box<dyn Error + Send + Sync>>;

/// Build the current package's cdylib for WASM, then embed its browser bindings.
/// On the nested WASM build this is a no-op, preventing recursive compilation.
pub fn build() -> Result<()> {
    Builder::default().build()
}

/// Configure Web-only Cargo features for an existing project.
#[derive(Default)]
pub struct Builder {
    features: Vec<String>,
    no_default_features: bool,
}

impl Builder {
    pub fn features(mut self, features: impl IntoIterator<Item = impl Into<String>>) -> Self {
        self.features = features.into_iter().map(Into::into).collect();
        self
    }

    pub fn no_default_features(mut self) -> Self {
        self.no_default_features = true;
        self
    }

    pub fn build(self) -> Result<()> {
        if env::var("CARGO_CFG_TARGET_ARCH").as_deref() == Ok("wasm32") {
            return Ok(());
        }
        let project = PathBuf::from(env::var_os("CARGO_MANIFEST_DIR").ok_or("run from build.rs")?);
        let output = PathBuf::from(env::var_os("OUT_DIR").ok_or("OUT_DIR unavailable")?);
        let manifest = project.join("Cargo.toml");
        let cargo = env::var_os("CARGO").unwrap_or_else(|| "cargo".into());
        let metadata = Command::new(&cargo)
            .args([
                "metadata",
                "--locked",
                "--no-deps",
                "--format-version",
                "1",
                "--manifest-path",
            ])
            .arg(&manifest)
            .output()?;
        if !metadata.status.success() {
            return Err(String::from_utf8_lossy(&metadata.stderr)
                .into_owned()
                .into());
        }
        let metadata: serde_json::Value = serde_json::from_slice(&metadata.stdout)?;
        let package = metadata["packages"]
            .as_array()
            .ok_or("missing Cargo packages")?
            .iter()
            .find(|p| {
                p["manifest_path"]
                    .as_str()
                    .is_some_and(|p| Path::new(p) == manifest)
            })
            .ok_or("current package missing from Cargo metadata")?;
        let package_id = package["id"].as_str().ok_or("missing Cargo package id")?;
        let mut command = Command::new(&cargo);
        command
            .current_dir(&project)
            .args([
                "build",
                "--locked",
                "--lib",
                "--target",
                "wasm32-unknown-unknown",
                "--message-format=json-render-diagnostics",
                "--manifest-path",
            ])
            .arg(&manifest)
            .arg("--target-dir")
            .arg(output.join("wasm-target"))
            // Let Cargo recompute flags for the WASM target, rather than inheriting native flags.
            .env_remove("CARGO_ENCODED_RUSTFLAGS")
            .env_remove("CARGO_MAKEFLAGS")
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit());
        if env::var("PROFILE").as_deref() == Ok("release") {
            command.arg("--release");
        }
        if self.no_default_features {
            command.arg("--no-default-features");
        }
        if !self.features.is_empty() {
            command.arg("--features").arg(self.features.join(","));
        }
        let mut child = command.spawn()?;
        let mut artifact = None;
        for line in BufReader::new(child.stdout.take().ok_or("Cargo stdout unavailable")?).lines() {
            let line = line?;
            let Ok(message) = serde_json::from_str::<serde_json::Value>(&line) else {
                continue;
            };
            if let Some(rendered) = message["message"]["rendered"].as_str() {
                eprint!("{rendered}");
            }
            if message["reason"] == "compiler-artifact" && message["package_id"] == package_id {
                artifact = message["filenames"]
                    .as_array()
                    .and_then(|files| {
                        files
                            .iter()
                            .filter_map(|p| p.as_str())
                            .find(|p| p.ends_with(".wasm"))
                    })
                    .map(PathBuf::from);
            }
        }
        if !child.wait()?.success() {
            return Err("WASM build failed; install wasm32-unknown-unknown and adapt native-only services for Web".into());
        }
        let wasm = artifact.ok_or("Web entry needs [lib] crate-type = [\"cdylib\", \"rlib\"] and an exported async start()")?;
        let site = output.join("site");
        if site.exists() {
            fs::remove_dir_all(&site)?;
        }
        fs::create_dir_all(&site)?;
        wasm_bindgen_cli_support::Bindgen::new()
            .input_path(wasm)
            .web(true)?
            .omit_default_module_path(false)
            .typescript(false)
            .out_name("app")
            .generate(&site)?;
        let mut files = Vec::new();
        collect(&site, &site, &mut files)?;
        files.sort();
        let mut embedded = String::from("&[\n");
        for (name, path) in files {
            embedded.push_str(&format!(
                "({name:?}, include_bytes!({:?}) as &'static [u8]),\n",
                path.to_str().ok_or("non-UTF-8 build path")?
            ));
        }
        embedded.push_str("]\n");
        fs::write(output.join("egui_web_sandbox.rs"), embedded)?;
        // Keep Cargo's default package-wide source tracking. Projects with inputs
        // outside this package can use their usual build.rs rerun declarations.
        Ok(())
    }
}

fn collect(root: &Path, directory: &Path, files: &mut Vec<(String, PathBuf)>) -> Result<()> {
    for entry in fs::read_dir(directory)? {
        let path = entry?.path();
        if path.is_dir() {
            collect(root, &path, files)?;
        } else {
            let name = path
                .strip_prefix(root)?
                .to_str()
                .ok_or("non-UTF-8 asset path")?
                .replace('\\', "/");
            files.push((name, path));
        }
    }
    Ok(())
}
