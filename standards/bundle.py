"""Build exact, immutable downstream payloads without evaluating target code."""

import copy
import hashlib
import json
import os
import re
import subprocess
from functools import lru_cache
from pathlib import Path, PurePosixPath

SOURCE = "Idea2Crate/standards"
LOCK = ".idea2crate/standards.lock.json"
CHECK = "Idea2Crate / standards"
SYNC_BRANCH = "codex/sync-standards"
VERSION = 2
SHA = re.compile(r"[0-9a-f]{40}\Z")


class PolicyError(Exception):
    """A contribution does not meet the policy."""


class OperationError(PolicyError):
    """Verification or synchronization could not finish safely."""


@lru_cache(maxsize=1)
def git_local_variables():
    # Git documents this list as the supported way for a hook to access a
    # foreign repository. In a worktree, GIT_DIR is an absolute path.
    result = subprocess.run(
        ["git", "rev-parse", "--local-env-vars"], capture_output=True, check=True
    )
    return frozenset(result.stdout.decode().splitlines())


def git_environment():
    return {
        key: value
        for key, value in os.environ.items()
        if key not in git_local_variables()
    }


def encode(value):
    return (
        json.dumps(value, ensure_ascii=False, indent=2, sort_keys=True) + "\n"
    ).encode()


def digest(data):
    return hashlib.sha256(data).hexdigest()


def local_matches(actual, expected):
    # Git may check UTF-8 managed text out as CRLF on Windows. Server checks
    # continue to compare exact Git blobs; only the local working copy varies.
    return actual.replace(b"\r\n", b"\n") == expected


def protected(path):
    """Reserve central paths while leaving project skills project-owned."""
    folded = path.casefold()
    name = PurePosixPath(path).name.casefold()
    skill = folded.removeprefix(".agents/skills/").split("/", 1)[0]
    return (
        name in {"agents.md", "agents.override.md"}
        or folded == ".pre-commit-config.yaml"
        or folded.startswith(".idea2crate/")
        or folded in {".agents/skills", ".idea2crate"}
        or (folded.startswith(".agents/skills/") and skill.startswith("idea2crate-"))
    )


def read_lock(data, repository):
    try:
        value = json.loads(data)
    except (ValueError, UnicodeError) as exc:
        raise PolicyError("Missing or invalid standards lockfile") from exc
    if isinstance(value, dict) and value.get("schema") != VERSION:
        raise PolicyError(
            "Unsupported bundle format version; reviewed enrollment required"
        )
    if (
        not isinstance(value, dict)
        or value.get("source") != SOURCE
        or value.get("repository") != repository
        or not isinstance(value.get("revision"), str)
        or not SHA.fullmatch(value["revision"])
        or not isinstance(value.get("files"), dict)
    ):
        raise PolicyError("Invalid standards identity or revision")
    return value


class GitSource:
    def __init__(self, root):
        self.root = Path(root)
        self._files = {}

    def git(self, *args):
        result = subprocess.run(
            ["git", "-C", str(self.root), *args],
            capture_output=True,
            check=False,
            env=git_environment(),
        )
        if result.returncode:
            raise OperationError("Cannot read the immutable central Git source")
        return result.stdout

    @property
    def head(self):
        return self.git("rev-parse", "HEAD").decode().strip()

    def approved(self, revision):
        if not isinstance(revision, str) or not SHA.fullmatch(revision):
            raise PolicyError("Expected a full central commit SHA")
        head = self.head
        result = subprocess.run(
            [
                "git",
                "-C",
                str(self.root),
                "merge-base",
                "--is-ancestor",
                revision,
                head,
            ],
            capture_output=True,
            check=False,
            env=git_environment(),
        )
        if result.returncode:
            raise PolicyError("Source revision is not an approved ancestor")

    def files(self, revision):
        self.approved(revision)
        if revision not in self._files:
            paths = (
                self.git("ls-tree", "-r", "--name-only", revision).decode().splitlines()
            )
            selected = [
                p
                for p in paths
                if p in {"AGENTS.md", "LICENSE", "profiles.json", "repositories.json"}
                or p.startswith(".agents/skills/idea2crate-")
            ]
            self._files[revision] = {
                p: self.git("show", revision + ":" + p) for p in selected
            }
        return self._files[revision]


def configuration(source, revision):
    value = json.loads(source.files(revision)["repositories.json"])
    if value.get("schema") != 1 or value.get("source") != SOURCE:
        raise PolicyError("Invalid central registry")
    repositories = value.get("repositories")
    if not isinstance(repositories, dict):
        raise PolicyError("Invalid repository enrollment list")
    revoked = value.get("revoked_revisions", {})
    if not isinstance(revoked, dict) or any(
        not SHA.fullmatch(sha) or not isinstance(reason, str) or not reason.strip()
        for sha, reason in revoked.items()
    ):
        raise PolicyError("Revocations require full source SHAs and nonempty reasons")
    for name, config in repositories.items():
        if (
            not re.fullmatch(r"Idea2Crate/[A-Za-z0-9_.-]+", name)
            or name == SOURCE
            or not isinstance(config, dict)
            or not re.fullmatch(r"[A-Za-z0-9_./-]+", config.get("branch", ""))
            or not config.get("profile")
            or (
                any(key in config for key in ("bootstrap_head", "bootstrap_base"))
                and not all(
                    isinstance(config.get(key), str) and SHA.fullmatch(config[key])
                    for key in ("bootstrap_head", "bootstrap_base")
                )
            )
        ):
            raise PolicyError("Invalid enrolled repository: " + name)
    return value


def registry(source, revision):
    return configuration(source, revision)["repositories"]


def require_active(source, revision):
    source.approved(revision)
    reason = (
        configuration(source, source.head).get("revoked_revisions", {}).get(revision)
    )
    if reason:
        raise PolicyError("Installed policy revision was revoked: " + reason)


def payload(source, revision, repository):
    source.approved(revision)
    files = source.files(revision)
    config = registry(source, revision).get(repository)
    if config is None:
        raise PolicyError("Repository is not enrolled at this revision")
    profiles = json.loads(files["profiles.json"])
    if profiles.get("bundle_version") != VERSION:
        raise PolicyError(
            "Unsupported bundle format version; reviewed enrollment required"
        )
    profile = profiles["profiles"].get(config["profile"])
    if not isinstance(profile, list):
        raise PolicyError("Unknown pre-commit profile")
    precommit = copy.deepcopy(profiles["base"])
    precommit["repos"].append(
        {
            "repo": "https://github.com/" + SOURCE,
            "rev": revision,
            "hooks": [{"id": "standards-integrity"}, {"id": "standards-secrets"}],
        }
    )
    if profile:
        precommit["repos"].extend(copy.deepcopy(profile))
    result = {
        p: data
        for p, data in files.items()
        if p == "AGENTS.md" or p.startswith(".agents/skills/idea2crate-")
    }
    result[".idea2crate/LICENSE"] = files["LICENSE"]
    result[".pre-commit-config.yaml"] = (
        b"# Generated by Idea2Crate/standards; edit the central source.\n"
        + encode(precommit)
    )
    result[LOCK] = encode(
        {
            "schema": VERSION,
            "source": SOURCE,
            "revision": revision,
            "repository": repository,
            "files": {p: digest(data) for p, data in result.items()},
        }
    )
    return result


def verify_tree(tree, expected):
    """tree maps Git paths to (mode, blob SHA); never trusts a supplied digest."""
    actual = {p for p in tree if protected(p)}
    if actual != set(expected):
        missing = sorted(set(expected) - actual)
        extra = sorted(actual - set(expected))
        raise PolicyError(
            "Managed paths differ; missing=%s extra=%s" % (missing, extra)
        )
    for path, data in expected.items():
        mode, sha = tree[path]
        blob = hashlib.sha1(
            b"blob " + str(len(data)).encode() + b"\0" + data
        ).hexdigest()
        if mode != "100644" or sha != blob:
            raise PolicyError("Managed file differs from central source: " + path)


def local_check(source, root):
    root = Path(root).resolve()
    data = (root / LOCK).read_bytes()
    lock = read_lock(data, json.loads(data)["repository"])
    revision = source.head
    if lock["revision"] != revision:
        raise PolicyError("Hook checkout does not match the installed policy revision")
    expected = payload(source, revision, lock["repository"])
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
    actual = {p for p in paths if p and protected(p)}
    if actual != set(expected):
        raise PolicyError("Managed file set differs from the installed policy")
    for path, data in expected.items():
        file = root / path
        for length in range(1, len(PurePosixPath(path).parts) + 1):
            if root.joinpath(*PurePosixPath(path).parts[:length]).is_symlink():
                raise PolicyError("Managed paths cannot use symlinks: " + path)
        if not local_matches(file.read_bytes(), data):
            raise PolicyError("Managed file differs from central source: " + path)
