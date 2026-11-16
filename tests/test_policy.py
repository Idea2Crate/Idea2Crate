import copy
import hashlib
import io
import json
import tempfile
import unittest
from contextlib import redirect_stderr
from pathlib import Path

from standards.automation import check_pull, validate
from standards.bundle import (
    LOCK,
    SOURCE,
    SYNC_BRANCH,
    PolicyError,
    payload,
    verify_tree,
)
from standards.secrets import scan

REPO = "Idea2Crate/Snip2Pin"
OLD = "a" * 40
NEW = "b" * 40
BASE = "c" * 40
HEAD = "d" * 40
BOT = 123456


def git_blob(data):
    return hashlib.sha1(b"blob " + str(len(data)).encode() + b"\0" + data).hexdigest()


class Source:
    def __init__(self):
        root = Path(__file__).resolve().parents[1]
        files = {
            p: (root / p).read_bytes()
            for p in ("AGENTS.md", "profiles.json", "repositories.json")
        }
        for path in (root / ".agents/skills").rglob("*"):
            if path.is_file():
                files[path.relative_to(root).as_posix()] = path.read_bytes()
        self.versions = {OLD: files, NEW: copy.deepcopy(files)}
        self.versions[NEW]["AGENTS.md"] += b"\nAn approved additional rule.\n"
        self.head = NEW

    def approved(self, revision):
        if revision not in self.versions:
            raise PolicyError("Unmerged central revision")

    def files(self, revision):
        self.approved(revision)
        return self.versions[revision]

    def bootstrap(self, sha):
        registry = json.loads(self.versions[NEW]["repositories.json"])
        registry["repositories"][REPO]["bootstrap_head"] = sha
        self.versions[NEW]["repositories.json"] = json.dumps(registry).encode()


class Api:
    def __init__(self, source, installed=True):
        self.blobs = {}
        self.base = BASE
        self.trees = {
            BASE: self.make_tree(payload(source, OLD, REPO) if installed else {})
        }
        self.trees[BASE]["app.rs"] = ("100644", git_blob(b"original"))
        self.trees[HEAD] = copy.deepcopy(self.trees[BASE])
        self.pr = {
            "number": 1,
            "node_id": "PR_test",
            "state": "open",
            "draft": False,
            "user": {"id": 7},
            "base": {"repo": {"full_name": REPO}, "ref": "main"},
            "head": {"sha": HEAD, "ref": "codex/change", "repo": {"full_name": REPO}},
        }
        self.checks = []
        self.merges = []
        self.pull_calls = 0
        self.change_head_at = None
        self.branch_calls = 0
        self.change_base_at = None

    def make_tree(self, files):
        result = {}
        for path, data in files.items():
            sha = git_blob(data)
            self.blobs[sha] = data
            result[path] = ("100644", sha)
        return result

    def tree(self, repository, revision):
        return copy.deepcopy(self.trees[revision])

    def blob(self, repository, sha):
        return self.blobs[sha]

    def branch(self, repository, name):
        self.branch_calls += 1
        if self.change_base_at == self.branch_calls:
            self.base = "e" * 40
        return {"object": {"sha": self.base}}

    def pull(self, repository, number):
        self.pull_calls += 1
        result = copy.deepcopy(self.pr)
        if self.change_head_at == self.pull_calls:
            result["head"]["sha"] = "f" * 40
        return result

    def check(self, repository, sha, conclusion, summary):
        self.checks.append((sha, conclusion, summary))

    def enable_auto_merge(self, node_id):
        self.merges.append(node_id)

    def set_sync(self, source):
        self.pr["user"]["id"] = BOT
        self.pr["head"]["ref"] = SYNC_BRANCH
        self.trees[HEAD] = {
            **self.trees[BASE],
            **self.make_tree(payload(source, NEW, REPO)),
        }


class PolicyTests(unittest.TestCase):
    def setUp(self):
        self.source = Source()
        self.api = Api(self.source)

    def rejected(self):
        with self.assertRaises(PolicyError):
            validate(self.api, self.source, REPO, self.api.pr, BOT)

    def test_normal_code_pr_keeps_approved_older_policy_and_is_not_auto_merged(self):
        self.api.trees[HEAD]["app.rs"] = ("100644", git_blob(b"changed"))
        self.assertTrue(check_pull(self.api, self.source, REPO, 1, BOT, merge=True))
        self.assertEqual(self.api.checks[-1][1], "success")
        self.assertEqual(self.api.merges, [])

    def test_edited_text_and_edited_digest_cannot_bless_each_other(self):
        tree = self.api.trees[HEAD]
        tree["AGENTS.md"] = ("100644", git_blob(b"weakened"))
        lock = json.loads(self.api.blob(REPO, tree[LOCK][1]))
        lock["files"]["AGENTS.md"] = hashlib.sha256(b"weakened").hexdigest()
        tree.update(self.api.make_tree({LOCK: json.dumps(lock).encode()}))
        self.rejected()

    def test_new_nested_or_override_instructions_are_rejected(self):
        for path in (
            "AGENTS.override.md",
            "src/AGENTS.md",
            "src/agents.MD",
            ".agents/skills/custom/SKILL.md",
        ):
            with self.subTest(path=path):
                api = Api(self.source)
                api.trees[HEAD][path] = ("100644", git_blob(b"new instructions"))
                with self.assertRaises(PolicyError):
                    validate(api, self.source, REPO, api.pr, BOT)

    def test_symlink_or_executable_managed_file_is_rejected(self):
        for mode in ("120000", "100755", "160000"):
            with self.subTest(mode=mode):
                tree = self.api.trees[HEAD]
                tree["AGENTS.md"] = (mode, self.api.trees[BASE]["AGENTS.md"][1])
                self.rejected()

    def test_exact_latest_payload_requires_authenticated_bot(self):
        self.api.trees[HEAD].update(self.api.make_tree(payload(self.source, NEW, REPO)))
        self.rejected()
        self.api.set_sync(self.source)
        self.assertTrue(check_pull(self.api, self.source, REPO, 1, BOT, merge=True))
        self.assertEqual(self.api.merges, ["PR_test"])

    def test_bot_cannot_sneak_unrelated_code_into_sync(self):
        self.api.set_sync(self.source)
        self.api.trees[HEAD]["app.rs"] = ("100644", git_blob(b"unreviewed"))
        self.rejected()

    def test_same_named_branch_on_fork_is_not_a_sync(self):
        self.api.set_sync(self.source)
        self.api.pr["head"]["repo"]["full_name"] = "someone/Snip2Pin"
        self.rejected()

    def test_unmerged_source_revision_is_rejected(self):
        tree = self.api.trees[HEAD]
        lock = json.loads(self.api.blob(REPO, tree[LOCK][1]))
        lock["revision"] = "0" * 40
        tree.update(self.api.make_tree({LOCK: json.dumps(lock).encode()}))
        self.rejected()

    def test_head_change_cannot_receive_success_for_new_head(self):
        self.api.change_head_at = 2
        self.assertFalse(check_pull(self.api, self.source, REPO, 1, BOT))
        self.assertEqual(self.api.checks[-1][0:2], (HEAD, "failure"))

    def test_base_change_is_not_approved(self):
        self.api.change_base_at = 2
        self.assertFalse(check_pull(self.api, self.source, REPO, 1, BOT))
        self.assertEqual(self.api.checks[-1][1], "failure")

    def test_bootstrap_needs_exact_central_approval(self):
        self.api = Api(self.source, installed=False)
        self.api.trees[HEAD].update(self.api.make_tree(payload(self.source, OLD, REPO)))
        self.api.trees[HEAD]["docs/project.md"] = (
            "100644",
            git_blob(b"preserved project rules"),
        )
        self.rejected()
        self.source.bootstrap(HEAD)
        result = validate(self.api, self.source, REPO, self.api.pr, BOT)
        self.assertFalse(result[2])
        self.source.bootstrap("e" * 40)
        self.rejected()

    def test_spent_bootstrap_does_not_authorize_later_policy_edits(self):
        self.source.bootstrap(HEAD)
        self.api.trees[HEAD].update(self.api.make_tree(payload(self.source, NEW, REPO)))
        self.rejected()

    def test_precommit_uses_immutable_source_and_keeps_project_checks(self):
        config = json.loads(
            payload(self.source, NEW, REPO)[".pre-commit-config.yaml"].split(b"\n", 1)[
                1
            ]
        )
        repo = next(
            x for x in config["repos"] if x["repo"] == "https://github.com/" + SOURCE
        )
        self.assertEqual(repo["rev"], NEW)
        local = next(x for x in config["repos"] if x["repo"] == "local")
        self.assertEqual(
            {x["entry"] for x in local["hooks"]},
            {"cargo xtask lint", "cargo deny --locked check"},
        )

    def test_root_instructions_are_identical_across_profiles(self):
        values = json.loads(self.source.versions[NEW]["repositories.json"])
        values["repositories"]["Idea2Crate/Example"] = {
            "profile": "base",
            "branch": "main",
        }
        self.source.versions[NEW]["repositories.json"] = json.dumps(values).encode()
        first = payload(self.source, NEW, REPO)
        other = payload(self.source, NEW, "Idea2Crate/Example")
        self.assertEqual(first["AGENTS.md"], other["AGENTS.md"])
        self.assertNotEqual(
            first[".pre-commit-config.yaml"], other[".pre-commit-config.yaml"]
        )

    def test_secret_guard_does_not_echo_credentials(self):
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "fixture.txt"
            fake = "gh" + "p_" + "a" * 36
            path.write_text(fake)
            errors = io.StringIO()
            with redirect_stderr(errors):
                self.assertEqual(scan([str(path)]), 1)
            self.assertNotIn(fake, errors.getvalue())
            path.write_text("ordinary source")
            self.assertEqual(scan([str(path)]), 0)

    def test_retired_skill_is_removed_from_expected_payload(self):
        skill = ".agents/skills/retired/SKILL.md"
        self.source.versions[OLD][skill] = b"retired"
        old = self.api.make_tree(payload(self.source, OLD, REPO))
        new = payload(self.source, NEW, REPO)
        with self.assertRaises(PolicyError):
            verify_tree(old, new)
        self.assertNotIn(skill, new)


if __name__ == "__main__":
    unittest.main()
