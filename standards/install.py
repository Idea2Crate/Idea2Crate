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
    require_active,
)


def require_clean_replacement(root, name):
    """Compare actual bytes and the index to HEAD, ignoring Git status hints."""

    def run(*args):
        return subprocess.run(
            ["git", "--literal-pathspecs", "-C", str(root), *args],
            capture_output=True,
            check=False,
            env=git_environment(),
        )

    file = root / name
    index = run("ls-files", "--stage", "-z", "--", name)
    if index.returncode:
        raise PolicyError("Cannot inspect replacement index")
    if not index.stdout and not file.exists():
        # An absent index/worktree path may be a staged deletion, not a new file.
        staged = run("diff", "--cached", "--quiet", "--no-ext-diff", "--", name)
        if staged.returncode:
            raise PolicyError("Refusing to replace a staged deletion: " + name)
        return
    head = run("ls-tree", "-z", "HEAD", "--", name)
    message = "Refusing to replace uncommitted or untracked managed file: " + name
    if (
        head.returncode
        or not head.stdout
        or not file.is_file()
        or index.stdout.count(b"\0") != 1
    ):
        raise PolicyError(message)
    indexed = index.stdout.partition(b"\t")[0].split()
    committed = head.stdout.partition(b"\t")[0].split()
    if (
        indexed[2] != b"0"
        or indexed[0] != committed[0]
        or indexed[1] != committed[2]
        or committed[1] != b"blob"
        or committed[0] not in {b"100644", b"100755"}
    ):
        raise PolicyError(message)
    original = run("cat-file", "blob", committed[2].decode())
    if original.returncode:
        raise PolicyError("Cannot inspect replacement baseline")
    actual = file.read_bytes()
    if actual != original.stdout and not local_matches(actual, original.stdout):
        raise PolicyError(message)


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
    require_active(source, revision)
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
    changed = [
        name
        for name in sorted(set(expected) - {LOCK}) + [LOCK]
        if not (root / name).exists()
        or not local_matches((root / name).read_bytes(), expected[name])
    ]
    retired = sorted(set(previous) - set(expected))
    replacing = changed + retired
    if not replacing:
        return []
    for name in replacing:
        require_clean_replacement(root, name)
    # Write the lock last: interruption leaves a detectable mismatch, never a
    # lockfile that blesses an incomplete update. The next run can be inspected.
    # Remove retired files before publishing the new lock as well.
    for name in retired:
        (root / name).unlink()
    for name in changed:
        file = root / name
        file.parent.mkdir(parents=True, exist_ok=True)
        file.write_bytes(expected[name])
    return changed + retired
