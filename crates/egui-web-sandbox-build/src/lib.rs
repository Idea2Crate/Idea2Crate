//! Build-script integration for a self-contained egui Web server executable.
use std::{
    collections::BTreeSet,
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
        let (wasm, workspace) = self.compile(
            &manifest,
            &output.join("wasm-target"),
            env::var("PROFILE").as_deref() == Ok("release"),
        )?;
        track_sources(&wasm, &manifest, &workspace, &output)?;
        let site = output.join("site");
        if site.exists() {
            fs::remove_dir_all(&site)?;
        }
        let mut embedded = String::from("&[\n");
        for (name, path) in generate(&wasm, &site)? {
            embedded.push_str(&format!(
                "({name:?}, include_bytes!({:?}) as &'static [u8]),\n",
                path.to_str().ok_or("non-UTF-8 build path")?
            ));
        }
        embedded.push_str("]\n");
        fs::write(output.join("egui_web_sandbox.rs"), embedded)?;
        Ok(())
    }

    fn compile(
        self,
        manifest: &Path,
        target_dir: &Path,
        release: bool,
    ) -> Result<(PathBuf, PathBuf)> {
        let manifest = manifest.canonicalize()?;
        let project = manifest
            .parent()
            .ok_or("manifest has no project directory")?;
        let cargo = env::var_os("CARGO").unwrap_or_else(|| "cargo".into());
        let metadata = Command::new(&cargo)
            .current_dir(project)
            .args([
                "metadata",
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
            .ok_or("use the Cargo.toml of the package containing the Web library")?;
        let package_id = package["id"].as_str().ok_or("missing Cargo package id")?;
        let workspace = PathBuf::from(
            metadata["workspace_root"]
                .as_str()
                .ok_or("missing Cargo workspace")?,
        );
        let mut command = Command::new(&cargo);
        command
            .current_dir(project)
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
            .args(["--package", package_id])
            .arg("--target-dir")
            .arg(target_dir)
            // A separate WASM build reuses the enclosing lockfile and computes its own flags.
            .env_remove("CARGO_ENCODED_RUSTFLAGS")
            .env_remove("CARGO_MAKEFLAGS")
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit());
        if release {
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
        let wasm = artifact.ok_or(
            "Web library did not produce a .wasm file; set [lib] crate-type = [\"cdylib\"]",
        )?;
        Ok((wasm, workspace))
    }
}

/// Generate browser bindings and serve a precompiled wasm-bindgen artifact.
#[cfg(feature = "cli")]
pub fn serve_wasm(wasm: &Path, address: std::net::SocketAddr) -> Result<()> {
    let mut output = web_bindgen(wasm)?.generate_output()?;
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
        let dependencies: std::collections::BTreeMap<_, _> = output
            .npm_dependencies()
            .iter()
            .map(|(name, (_, version))| (name, version))
            .collect();
        let manifest = serde_json::json!({ "dependencies": dependencies });
        assets.push(("package.json".into(), serde_json::to_vec_pretty(&manifest)?));
    }

    drop(output);
    let server = egui_web_sandbox::server::Server::bind_assets_at(assets, address)?;
    println!("{}", server.url());
    server.run();
    Ok(())
}

fn track_sources(wasm: &Path, manifest: &Path, workspace: &Path, output: &Path) -> Result<()> {
    // Follow the transitive local sources and included assets in Cargo's depfile.
    let dependencies = fs::read_to_string(wasm.with_extension("d"))?;
    // The output filename is irrelevant; normalize its rule target so a
    // Windows drive colon is not mistaken for the dependency separator.
    let (_, dependencies) = dependencies
        .split_once(": ")
        .ok_or("invalid Cargo depfile rule")?;
    let dependencies = format!("web: {dependencies}");
    let dependencies = depfile::parse(&dependencies)
        .map_err(|position| format!("invalid Cargo depfile at byte {position}"))?;
    let project = manifest
        .parent()
        .ok_or("manifest has no project directory")?;
    let mut inputs = BTreeSet::from([
        manifest.to_path_buf(),
        workspace.join("Cargo.toml"),
        workspace.join("Cargo.lock"),
    ]);
    for (_, dependencies) in dependencies.iter() {
        for path in dependencies {
            let path = project.join(path.as_ref());
            if path.starts_with(output) {
                continue;
            }
            if let Some(root) = path
                .ancestors()
                .find(|path| path.join("Cargo.toml").is_file())
            {
                inputs.insert(root.join("Cargo.toml"));
            }
            inputs.insert(path);
        }
    }
    for input in inputs {
        println!("cargo:rerun-if-changed={}", input.display());
    }
    Ok(())
}

fn web_bindgen(wasm: &Path) -> Result<wasm_bindgen_cli_support::Bindgen> {
    let mut bindgen = wasm_bindgen_cli_support::Bindgen::new();
    bindgen
        .input_path(wasm)
        .web(true)?
        .omit_default_module_path(false)
        .typescript(false)
        .out_name("app");
    Ok(bindgen)
}

fn generate(wasm: &Path, site: &Path) -> Result<Vec<(String, PathBuf)>> {
    fs::create_dir_all(site)?;
    web_bindgen(wasm)?.generate(site)?;
    let mut files = Vec::new();
    collect(site, site, &mut files)?;
    files.sort();
    Ok(files)
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
