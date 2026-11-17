"""Build this Web entry and obtain browser assets from its resolved sandbox crate."""

import argparse
import json
from pathlib import Path
import shutil
import subprocess


def main():
    project = Path(__file__).resolve().parent
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--out-dir", type=Path, default=project / ".build/site")
    parser.add_argument("--locked", action="store_true")
    args = parser.parse_args()
    manifest = ["--manifest-path", str(project / "Cargo.toml")]
    locked = ["--locked"] if args.locked else []
    metadata = json.loads(
        subprocess.check_output(
            [
                "cargo",
                "metadata",
                *manifest,
                *locked,
                "--format-version",
                "1",
                "--filter-platform",
                "wasm32-unknown-unknown",
            ],
            cwd=project,
        )
    )
    package = next(
        p
        for p in metadata["packages"]
        if Path(p["manifest_path"]) == project / "Cargo.toml"
    )
    sandbox = [p for p in metadata["packages"] if p["name"] == "egui-web-sandbox"]
    bindings = [p for p in metadata["packages"] if p["name"] == "wasm-bindgen"]
    if len(sandbox) != 1 or len(bindings) != 1:
        raise SystemExit("The Web entry needs one sandbox and one wasm-bindgen version")
    version = bindings[0]["version"]
    install = f"cargo install wasm-bindgen-cli --version {version} --locked"
    try:
        actual = subprocess.check_output(
            ["wasm-bindgen", "--version"], text=True
        ).strip()
    except FileNotFoundError:
        raise SystemExit(f"Install matching tools: {install}") from None
    if actual != f"wasm-bindgen {version}":
        raise SystemExit(f"Install matching tools: {install}")
    target = next(t for t in package["targets"] if "cdylib" in t["crate_types"])
    subprocess.run(
        [
            "cargo",
            "build",
            *manifest,
            *locked,
            "--package",
            package["name"],
            "--target",
            "wasm32-unknown-unknown",
        ],
        cwd=project,
        check=True,
    )
    binary = (
        Path(metadata["target_directory"])
        / "wasm32-unknown-unknown/debug"
        / f"{target['name']}.wasm"
    )
    output = args.out_dir.resolve()
    subprocess.run(
        [
            "wasm-bindgen",
            str(binary),
            "--target",
            "web",
            "--no-typescript",
            "--out-name",
            "app",
            "--out-dir",
            str(output),
        ],
        check=True,
    )
    assets = Path(sandbox[0]["manifest_path"]).parent / "browser"
    for name in ("launch.mjs", "client.mjs"):
        shutil.copyfile(assets / name, output / name)
    shutil.copyfile(project / "index.html", output / "index.html")
    shutil.copyfile(project / "LICENSE", output / "LICENSE.txt")
    print(f"Web app ready: {output}")


if __name__ == "__main__":
    try:
        main()
    except subprocess.CalledProcessError as error:
        raise SystemExit(error.returncode) from None
