"""Apply a computed bundle locally, preserving project docs and unrelated work."""

import subprocess
from pathlib import Path, PurePosixPath

from .bundle import (
    LOCK,
    PolicyError,
    git_environment,
    local_matches,
    payload,
    protected,
    read_lock,
)


def apply_local(source, target, repository, revision):
    root = Path(target).resolve()
    git = subprocess.run(
        ["git", "-C", str(root), "rev-parse", "--show-toplevel"],
        capture_output=True,
        check=True,
        env=git_environment(),
    )
    if Path(git.stdout.decode().strip()).resolve() != root:
        raise PolicyError("Target must be the repository root")
    if not (root / "docs/project.md").is_file():
        raise PolicyError(
            "Preserve project instructions in docs/project.md before enrollment"
        )
    expected = payload(source, revision, repository)
    previous = {}
    if (root / LOCK).exists():
        old = read_lock((root / LOCK).read_bytes(), repository)
        previous = payload(source, old["revision"], repository)
        for name, content in previous.items():
            if not local_matches((root / name).read_bytes(), content):
                raise PolicyError("Refusing to overwrite edited managed file: " + name)
    paths = (
        subprocess.run(
            [
                "git",
                "-C",
                str(root),
                "ls-files",
                "--cached",
                "--others",
                "--exclude-standard",
                "-z",
            ],
            capture_output=True,
            check=True,
            env=git_environment(),
        )
        .stdout.decode()
        .split("\0")
    )
    allowed = set(previous) | set(expected)
    for name in paths:
        if name and protected(name) and name not in allowed:
            raise PolicyError(
                "Move local/override instructions out of managed paths first: " + name
            )
    for name in allowed:
        parts = PurePosixPath(name).parts
        for length in range(1, len(parts) + 1):
            if root.joinpath(*parts[:length]).is_symlink():
                raise PolicyError("Refusing to write through a symlink: " + name)
    changed = []
    # Write the lock last: interruption leaves a detectable mismatch, never a
    # lockfile that blesses an incomplete update. The next run can be inspected.
    for name in sorted(set(expected) - {LOCK}) + [LOCK]:
        file = root / name
        if not file.exists() or not local_matches(file.read_bytes(), expected[name]):
            file.parent.mkdir(parents=True, exist_ok=True)
            file.write_bytes(expected[name])
            changed.append(name)
    for name in sorted(set(previous) - set(expected)):
        (root / name).unlink()
        changed.append(name)
    return changed
