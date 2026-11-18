use crate::{Result, cli::PackOptions};
use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeMap,
    fs::{self, File},
    io::{self, Cursor, Read, Write},
    path::Path,
};
use zip::{CompressionMethod, ZipArchive, ZipWriter, write::SimpleFileOptions};

const FORMAT_VERSION: u32 = 1;
const MANIFEST: &str = "version.json";

#[derive(Deserialize, Serialize)]
struct Version {
    format_version: u32,
    entry: String,
    packer_version: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    app_version: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    wasm_bindgen_version: Option<String>,
}

pub struct Bundle {
    pub entry: String,
    pub assets: BTreeMap<String, Vec<u8>>,
}

pub fn pack(options: PackOptions) -> Result<()> {
    let entry = options
        .entry
        .file_name()
        .and_then(|name| name.to_str())
        .filter(|name| !name.contains('\\'))
        .ok_or("expected a UTF-8 filename for the generated JS entry")?;
    let stem = entry.strip_suffix(".js").expect("validated JS entry");
    let directory = options.entry.parent().unwrap_or(Path::new(""));
    let wasm = format!("{stem}_bg.wasm");
    let glue = format!("{stem}_bg.js");
    let mut assets = BTreeMap::from([
        (entry.to_owned(), read_asset(&options.entry)?),
        (wasm.clone(), read_asset(&directory.join(&wasm))?),
    ]);
    match fs::symlink_metadata(directory.join(&glue)) {
        Ok(_) => {
            let bytes = read_asset(&directory.join(&glue))?;
            assets.insert(glue, bytes);
        }
        Err(error) if error.kind() == io::ErrorKind::NotFound => {}
        Err(error) => return Err(error.into()),
    }
    let snippets = directory.join("snippets");
    match fs::symlink_metadata(&snippets) {
        Ok(metadata) if metadata.is_dir() => read_snippets(&snippets, "snippets", &mut assets)?,
        Ok(_) => return Err(format!("expected a directory at {}", snippets.display()).into()),
        Err(error) if error.kind() == io::ErrorKind::NotFound => {}
        Err(error) => return Err(error.into()),
    }

    let version = Version {
        format_version: FORMAT_VERSION,
        entry: entry.to_owned(),
        packer_version: env!("CARGO_PKG_VERSION").to_owned(),
        app_version: options.app_version,
        wasm_bindgen_version: options.wasm_bindgen_version,
    };
    assets.insert(MANIFEST.into(), serde_json::to_vec_pretty(&version)?);

    let mut archive = ZipWriter::new(Cursor::new(Vec::new()));
    let stored = SimpleFileOptions::default().compression_method(CompressionMethod::Stored);
    for (name, bytes) in assets {
        archive.start_file(name, stored)?;
        archive.write_all(&bytes)?;
    }
    fs::write(options.output, archive.finish()?.into_inner())?;
    Ok(())
}

pub fn open(path: &Path) -> Result<Bundle> {
    let mut archive = ZipArchive::new(File::open(path)?)?;
    let manifest = archive.by_name(MANIFEST)?;
    let metadata: serde_json::Value = serde_json::from_reader(manifest)?;
    let format_version = metadata["format_version"]
        .as_u64()
        .ok_or("version.json must contain an integer format_version")?;
    if format_version != u64::from(FORMAT_VERSION) {
        return Err(format!(
            "unsupported bundle format {format_version}; this CLI supports {FORMAT_VERSION}"
        )
        .into());
    }
    let metadata: Version = serde_json::from_value(metadata)?;
    let entry = metadata.entry;
    let stem = entry
        .strip_suffix(".js")
        .filter(|stem| !stem.is_empty() && !stem.contains(['/', '\\']))
        .ok_or("version.json entry must name a root-level JS file")?;
    let wasm = format!("{stem}_bg.wasm");
    let glue = format!("{stem}_bg.js");
    let mut assets = BTreeMap::new();
    for index in 0..archive.len() {
        let mut file = archive.by_index(index)?;
        let name = file.name()?.into_owned();
        if file.enclosed_name().is_none()
            || !file.is_file()
            || file.compression() != CompressionMethod::Stored
            || (name != entry
                && name != wasm
                && name != glue
                && name != MANIFEST
                && !(name.starts_with("snippets/") && is_js(Path::new(&name))))
        {
            return Err(format!("unexpected bundle entry {name:?}").into());
        }
        let mut bytes = Vec::new();
        file.read_to_end(&mut bytes)?;
        assets.insert(name, bytes);
    }
    for required in [&entry, &wasm] {
        if !assets.contains_key(required) {
            return Err(format!("bundle is missing {required}").into());
        }
    }
    Ok(Bundle { entry, assets })
}

fn read_asset(path: &Path) -> Result<Vec<u8>> {
    let error_at = |error| format!("could not read {}: {error}", path.display());
    if !fs::symlink_metadata(path).map_err(error_at)?.is_file() {
        return Err(format!("expected a regular file at {}", path.display()).into());
    }
    fs::read(path).map_err(|error| error_at(error).into())
}

fn read_snippets(path: &Path, prefix: &str, assets: &mut BTreeMap<String, Vec<u8>>) -> Result<()> {
    for entry in fs::read_dir(path)? {
        let entry = entry?;
        let kind = entry.file_type()?;
        let path = entry.path();
        if kind.is_dir() || (kind.is_file() && is_js(&path)) {
            let name = entry.file_name();
            let name = name
                .to_str()
                .filter(|name| !name.contains('\\'))
                .ok_or("expected a UTF-8 filename in snippets/")?;
            let name = format!("{prefix}/{name}");
            if kind.is_dir() {
                read_snippets(&path, &name, assets)?;
            } else {
                assets.insert(name, read_asset(&path)?);
            }
        }
    }
    Ok(())
}

fn is_js(path: &Path) -> bool {
    matches!(
        path.extension().and_then(|value| value.to_str()),
        Some("js" | "mjs")
    )
}
