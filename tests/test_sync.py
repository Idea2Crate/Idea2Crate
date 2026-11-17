import copy
import json
import unittest
from unittest.mock import patch

from test_policy import Api, BASE, BOT, NEW, OLD, REPO, Source, git_blob
from standards.automation import check_pull, reconcile, sync_repository
from standards.bundle import (
    LOCK,
    SYNC_BRANCH,
    OperationError,
    PolicyError,
    payload,
    verify_tree,
)


class Remote(Api):
    def __init__(self, source):
        super().__init__(source)
        self.open_pr = None
        self.ref = None
        self.commits = {
            BASE: {"sha": BASE, "tree": {"sha": "initial-tree"}, "parents": []}
        }
        self.git_trees = {"initial-tree": copy.deepcopy(self.trees[BASE])}
        self.writes = []
        self.sequence = 1
        self.race = False

    def pages(self, path):
        return [] if self.open_pr is None else [copy.deepcopy(self.open_pr)]

    def pull(self, repository, number):
        return copy.deepcopy(self.open_pr)

    def enable_auto_merge(self, node_id):
        super().enable_auto_merge(node_id)
        self.open_pr["auto_merge"] = {"enabled_by": {"id": BOT}}

    def request(self, method, path, data=None, allow_missing=False):
        prefix = "/repos/" + REPO
        relative = path.removeprefix(prefix)
        if method == "GET" and relative.startswith("/git/ref/heads/"):
            if self.ref is None and not allow_missing:
                raise PolicyError("Missing ref")
            return None if self.ref is None else {"object": {"sha": self.ref}}
        if method == "GET" and relative.startswith("/git/commits/"):
            return self.commits[relative.split("/")[-1]]
        self.writes.append((method, path, copy.deepcopy(data)))
        if method == "POST" and relative == "/git/trees":
            tree = copy.deepcopy(self.git_trees[data["base_tree"]])
            for entry in data["tree"]:
                if entry.get("sha", True) is None:
                    tree.pop(entry["path"], None)
                else:
                    tree.update(
                        self.make_tree({entry["path"]: entry["content"].encode()})
                    )
            sha = "tree-" + str(self.sequence)
            self.git_trees[sha] = tree
            return {"sha": sha}
        if method == "POST" and relative == "/git/commits":
            sha = str(self.sequence).zfill(40)
            self.sequence += 1
            self.commits[sha] = {
                "sha": sha,
                "tree": {"sha": data["tree"]},
                "parents": [{"sha": p} for p in data["parents"]],
            }
            self.trees[sha] = copy.deepcopy(self.git_trees[data["tree"]])
            return {"sha": sha}
        if method == "POST" and relative == "/git/refs":
            self.ref = data["sha"]
            return {}
        if method == "PATCH" and relative.startswith("/git/refs/heads/"):
            if data.get("force") is not False:
                raise AssertionError("Bot must never force-push")
            if self.race:
                raise PolicyError("Concurrent commit prevents fast-forward")
            self.ref = data["sha"]
            if self.open_pr:
                self.open_pr["head"]["sha"] = self.ref
                self.open_pr["auto_merge"] = None
            return {}
        if method == "POST" and relative == "/pulls":
            if self.open_pr:
                raise AssertionError("Duplicate sync PR")
            self.open_pr = copy.deepcopy(self.pr)
            self.open_pr["head"].update({"sha": self.ref, "ref": SYNC_BRANCH})
            self.open_pr["user"]["id"] = BOT
            return self.open_pr
        raise AssertionError((method, path, data))

    def move_main(self):
        self.base = "e" * 40
        self.trees[self.base] = copy.deepcopy(self.trees[BASE])
        self.trees[self.base]["new-code.rs"] = (
            "100644",
            git_blob(b"unrelated main change"),
        )
        self.git_trees["moved-tree"] = copy.deepcopy(self.trees[self.base])
        self.commits[self.base] = {
            "sha": self.base,
            "tree": {"sha": "moved-tree"},
            "parents": [{"sha": BASE}],
        }

    def is_ancestor(self, repository, ancestor, descendant):
        pending = [descendant]
        visited = set()
        while pending:
            sha = pending.pop()
            if sha == ancestor:
                return True
            if sha not in visited:
                visited.add(sha)
                pending.extend(p["sha"] for p in self.commits[sha]["parents"])
        return False


class SyncTests(unittest.TestCase):
    def setUp(self):
        self.source = Source()
        self.api = Remote(self.source)

    def test_sync_is_exact_preserves_app_and_is_idempotent(self):
        number = sync_repository(self.api, self.source, REPO, BOT)
        self.assertEqual(number, 1)
        verify_tree(self.api.trees[self.api.ref], payload(self.source, NEW, REPO))
        self.assertEqual(
            self.api.trees[self.api.ref]["app.rs"], self.api.trees[BASE]["app.rs"]
        )
        writes = len(self.api.writes)
        sync_repository(self.api, self.source, REPO, BOT)
        self.assertEqual(len(self.api.writes), writes)
        self.assertEqual(self.api.merges, ["PR_test"])

    def test_retired_files_are_deleted_during_sync(self):
        path = ".agents/skills/idea2crate-retired/SKILL.md"
        self.source.versions[OLD][path] = b"old"
        self.api = Remote(self.source)
        sync_repository(self.api, self.source, REPO, BOT)
        self.assertNotIn(path, self.api.trees[self.api.ref])

    def test_moved_base_rebuilds_without_losing_code_or_force_push(self):
        sync_repository(self.api, self.source, REPO, BOT)
        first = self.api.ref
        self.api.move_main()
        sync_repository(self.api, self.source, REPO, BOT)
        self.assertNotEqual(self.api.ref, first)
        self.assertIn("new-code.rs", self.api.trees[self.api.ref])

    def test_mixed_bot_branch_is_never_overwritten(self):
        sync_repository(self.api, self.source, REPO, BOT)
        first = self.api.ref
        feature, mixed = "8" * 40, "9" * 40
        self.api.trees[feature] = copy.deepcopy(self.api.trees[BASE])
        self.api.trees[feature]["injected.rs"] = (
            "100644",
            git_blob(b"unexpected"),
        )
        self.api.trees[mixed] = {
            **self.api.trees[feature],
            **self.api.make_tree(payload(self.source, NEW, REPO)),
        }
        self.api.commits[mixed] = {"parents": [{"sha": first}, {"sha": feature}]}
        self.api.ref = mixed
        self.api.open_pr["head"]["sha"] = mixed
        writes = len(self.api.writes)
        with self.assertRaises(PolicyError):
            sync_repository(self.api, self.source, REPO, BOT)
        self.assertEqual(len(self.api.writes), writes)
        self.assertEqual(self.api.ref, mixed)
        self.assertIn("injected.rs", self.api.trees[self.api.ref])

    def test_project_skills_and_license_survive_sync(self):
        source = Source()
        api = Remote(source)
        name = ".agents/skills/project-build/SKILL.md"
        entry = ("100644", git_blob(b"project skill"))
        api.trees[BASE][name] = entry
        api.git_trees["initial-tree"][name] = entry
        sync_repository(api, source, REPO, BOT)
        self.assertEqual(api.trees[api.ref][name], entry)
        notice = api.trees[api.ref][".idea2crate/LICENSE"][1]
        self.assertEqual(api.blob(REPO, notice), source.files(NEW)["LICENSE"])

    def test_revocation_blocks_old_pr_but_allows_sync_repair(self):
        config = json.loads(self.source.versions[NEW]["repositories.json"])
        config["revoked_revisions"] = {OLD: "Unsafe historical policy"}
        self.source.versions[NEW]["repositories.json"] = json.dumps(config).encode()
        ordinary = Api(self.source)
        self.assertFalse(check_pull(ordinary, self.source, REPO, 1, BOT))
        sync_repository(self.api, self.source, REPO, BOT)
        self.assertEqual(self.api.checks[-1][1], "success")

    def test_reconciliation_distinguishes_rejection_from_operational_failure(self):
        self.api.open_pr = copy.deepcopy(self.api.pr)
        self.api.trees[self.api.pr["head"]["sha"]]["AGENTS.md"] = (
            "100644",
            git_blob(b"edited"),
        )
        self.assertEqual(reconcile(self.api, self.source, BOT), [])
        self.assertEqual(self.api.checks[-1][1], "failure")
        with patch.object(self.api, "tree", side_effect=OperationError("Unavailable")):
            self.assertTrue(reconcile(self.api, self.source, BOT))
        self.api.open_pr = None

        def retarget(repository, number):
            pr = copy.deepcopy(self.api.open_pr)
            pr["base"]["ref"] = "staging"
            return pr

        with patch.object(self.api, "pull", side_effect=retarget):
            self.assertTrue(reconcile(self.api, self.source, BOT, sync=True))

    def test_concurrent_push_causes_failure_without_force_overwrite(self):
        sync_repository(self.api, self.source, REPO, BOT)
        old = self.api.ref
        self.api.move_main()
        self.api.race = True
        with self.assertRaises(PolicyError):
            sync_repository(self.api, self.source, REPO, BOT)
        self.assertEqual(self.api.ref, old)

    def test_enrollment_is_not_silently_overwritten(self):
        del self.api.trees[BASE][LOCK]
        with self.assertRaises(PolicyError):
            sync_repository(self.api, self.source, REPO, BOT)
        self.assertEqual(self.api.writes, [])


if __name__ == "__main__":
    unittest.main()
