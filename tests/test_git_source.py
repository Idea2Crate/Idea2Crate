import os
import subprocess
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch

from standards.bundle import (
    GitSource,
    PolicyError,
    git_environment,
    local_check,
    payload,
)
from standards.install import apply_local
from test_policy import NEW, REPO, Source


def git(path, *args):
    return (
        subprocess.run(
            ["git", "-C", str(path), *args],
            check=True,
            capture_output=True,
            env=git_environment(),
        )
        .stdout.decode()
        .strip()
    )


class GitSourceTests(unittest.TestCase):
    def test_approved_ancestry_and_offline_local_hook(self):
        root = Path(__file__).resolve().parents[1]
        with tempfile.TemporaryDirectory() as directory:
            source = Path(directory) / "source"
            target = Path(directory) / "target"
            source.mkdir()
            target.mkdir()

            git(source, "init", "-b", "main")
            git(source, "config", "user.name", "Test")
            git(source, "config", "user.email", "test@example.invalid")
            for path in ("AGENTS.md", "LICENSE", "profiles.json", "repositories.json"):
                (source / path).write_bytes((root / path).read_bytes())
            git(source, "add", ".")
            git(source, "commit", "-m", "test: source")
            approved = git(source, "rev-parse", "HEAD")
            git(source, "checkout", "-b", "unmerged")
            (source / "unmerged.txt").write_text("unmerged")
            git(source, "add", ".")
            git(source, "commit", "-m", "test: unmerged")
            unmerged = git(source, "rev-parse", "HEAD")
            git(source, "checkout", "main")
            central = GitSource(source)
            central.approved(approved)
            with self.assertRaises(PolicyError):
                central.approved(unmerged)
            git(target, "init", "-b", "main")
            with self.assertRaises(PolicyError):
                apply_local(central, target, "Idea2Crate/Snip2Pin", approved)

            (target / "docs").mkdir()
            (target / "docs/project.md").write_text("Preserved project rules")
            (target / "app.rs").write_text("unrelated work")
            changed = apply_local(central, target, "Idea2Crate/Snip2Pin", approved)
            self.assertEqual(
                set(changed), set(payload(central, approved, "Idea2Crate/Snip2Pin"))
            )
            self.assertEqual((target / "app.rs").read_text(), "unrelated work")
            self.assertEqual(
                apply_local(central, target, "Idea2Crate/Snip2Pin", approved), []
            )
            git(target, "add", ".")
            local_check(central, target)
            for name, content in payload(
                central, approved, "Idea2Crate/Snip2Pin"
            ).items():
                (target / name).write_bytes(content.replace(b"\n", b"\r\n"))
            local_check(central, target)
            self.assertEqual(
                apply_local(central, target, "Idea2Crate/Snip2Pin", approved), []
            )
            # This is the environment Git exports to hooks in linked worktrees.
            # Foreign source reads must not silently use the target's Git dir.
            with patch.dict(
                os.environ,
                {
                    "GIT_DIR": str(target / ".git"),
                    "GIT_WORK_TREE": str(target),
                    "GIT_INDEX_FILE": str(target / ".git/index"),
                },
            ):
                self.assertEqual(central.head, approved)
                local_check(central, target)
            (target / "AGENTS.md").write_text("edited")
            with self.assertRaises(PolicyError):
                local_check(central, target)
            with self.assertRaises(PolicyError):
                apply_local(central, target, "Idea2Crate/Snip2Pin", approved)

    def test_enrollment_preserves_local_edits_and_migrates_only_clean_files(self):
        source = Source()
        for state in (
            "untracked",
            "unstaged",
            "staged",
            "assume-unchanged",
            "skip-worktree",
            "deleted",
            "clean",
        ):
            with self.subTest(state=state), tempfile.TemporaryDirectory() as directory:
                target = Path(directory)
                git(target, "init", "-b", "main")
                git(target, "config", "user.name", "Test")
                git(target, "config", "user.email", "test@example.invalid")
                (target / "docs").mkdir()
                (target / "docs/project.md").write_text("Preserved project rules")
                instructions = target / "AGENTS.md"
                instructions.write_text("Original project rules")
                if state != "untracked":
                    git(target, "add", ".")
                    git(target, "commit", "-m", "test: original rules")
                if state in {"assume-unchanged", "skip-worktree"}:
                    git(target, "update-index", "--" + state, "AGENTS.md")
                if state in {"unstaged", "staged", "assume-unchanged", "skip-worktree"}:
                    instructions.write_text("Uncommitted project rules")
                if state == "staged":
                    git(target, "add", "AGENTS.md")
                if state == "deleted":
                    git(target, "rm", "AGENTS.md")
                before = instructions.read_bytes() if instructions.exists() else None
                if state == "clean":
                    apply_local(source, target, REPO, NEW)
                    self.assertEqual(
                        instructions.read_bytes(), source.files(NEW)["AGENTS.md"]
                    )
                    self.assertEqual(
                        (target / "docs/project.md").read_text(),
                        "Preserved project rules",
                    )
                else:
                    with self.assertRaises(PolicyError):
                        apply_local(source, target, REPO, NEW)
                    self.assertEqual(
                        instructions.read_bytes() if instructions.exists() else None,
                        before,
                    )
                    self.assertFalse(
                        (target / ".idea2crate/standards.lock.json").exists()
                    )


if __name__ == "__main__":
    unittest.main()
