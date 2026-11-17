"""Trusted orchestration: target PR code is never checked out or executed."""

from urllib.parse import quote

from .bundle import (
    LOCK,
    SOURCE,
    SYNC_BRANCH,
    OperationError,
    PolicyError,
    payload,
    read_lock,
    registry,
    require_active,
    verify_tree,
)


def differences(before, after):
    return {p for p in set(before) | set(after) if before.get(p) != after.get(p)}


def same_pull(left, right):
    def identity(pr):
        return (
            pr["state"],
            pr["user"]["id"],
            pr.get("draft", False),
            pr["base"]["repo"]["full_name"],
            pr["base"]["ref"],
            (pr["head"]["repo"] or {}).get("full_name"),
            pr["head"]["ref"],
            pr["head"]["sha"],
        )

    return right["state"] == "open" and identity(left) == identity(right)


def validate(api, source, repository, pr, bot_id):
    config = registry(source, source.head)[repository]
    if (
        pr["state"] != "open"
        or pr["base"]["repo"]["full_name"] != repository
        or pr["base"]["ref"] != config["branch"]
    ):
        raise PolicyError("PR must target the enrolled repository and branch")
    base_sha = api.branch(repository, config["branch"])["object"]["sha"]
    before = api.tree(repository, base_sha)
    after = api.tree(repository, pr["head"]["sha"])
    if LOCK not in after:
        raise PolicyError("Repository must complete standards enrollment first")
    lock = read_lock(api.blob(repository, after[LOCK][1]), repository)
    require_active(source, lock["revision"])
    expected = payload(source, lock["revision"], repository)
    verify_tree(after, expected)
    # Explicit enrollment approval is bound to both target head and current base.
    # It may also authorize a reviewed format migration without interpreting it.
    bootstrap = (
        config.get("bootstrap_head") == pr["head"]["sha"]
        and config.get("bootstrap_base") == base_sha
    )
    changed = differences(before, after)
    if bootstrap:
        if "docs/project.md" not in after:
            raise PolicyError(
                "Enrollment must preserve project instructions in docs/project.md"
            )
        return base_sha, "Centrally approved enrollment commit", False
    if LOCK not in before:
        raise PolicyError(
            "Initial enrollment requires central approval of the exact head SHA"
        )
    old_lock = read_lock(api.blob(repository, before[LOCK][1]), repository)
    old_expected = payload(source, old_lock["revision"], repository)
    verify_tree(before, old_expected)
    policy_changes = changed & (set(old_expected) | set(expected))
    if not policy_changes:
        return base_sha, "Managed files match the installed approved revision", False
    if (
        pr["user"]["id"] != bot_id
        or pr["head"]["ref"] != SYNC_BRANCH
        or pr["head"]["repo"] is None
        or pr["head"]["repo"]["full_name"] != repository
        or changed != policy_changes
    ):
        raise PolicyError(
            "Only an authenticated, policy-only sync PR may change managed files"
        )
    # Latest approved payload, not a checksum/revision selected by the PR author.
    verify_tree(after, payload(source, source.head, repository))
    return (
        base_sha,
        "Exact synchronization of the latest approved central payload",
        True,
    )


def check_pull(api, source, repository, number, bot_id, merge=False):
    pr = api.pull(repository, number)
    sha = pr["head"]["sha"]
    try:
        base_sha, summary, sync = validate(api, source, repository, pr, bot_id)
        fresh = api.pull(repository, number)
        if not same_pull(pr, fresh):
            raise OperationError(
                "PR changed during validation; a new check is required"
            )
        branch = registry(source, source.head)[repository]["branch"]
        if api.branch(repository, branch)["object"]["sha"] != base_sha:
            raise OperationError(
                "Base branch changed during validation; retry on the current base"
            )
        api.check(repository, sha, "success", summary)
        if merge and sync and not fresh.get("draft") and not fresh.get("auto_merge"):
            # GitHub cannot atomically bind this mutation to a PR snapshot.
            # Recheck its full identity and the enrolled base before enabling it.
            if (
                not same_pull(pr, api.pull(repository, number))
                or api.branch(repository, branch)["object"]["sha"] != base_sha
            ):
                raise OperationError("PR changed before auto-merge could be enabled")
            api.enable_auto_merge(pr["node_id"])
    except PolicyError as error:
        api.check(repository, sha, "failure", str(error))
        if isinstance(error, OperationError):
            raise
        return False
    return True


def sync_repository(api, source, repository, bot_id):
    config = registry(source, source.head)[repository]
    base_sha = api.branch(repository, config["branch"])["object"]["sha"]
    before = api.tree(repository, base_sha)
    if LOCK not in before:
        raise PolicyError(
            "Enroll the repository through its reviewed migration PR before syncing"
        )
    old_lock = read_lock(api.blob(repository, before[LOCK][1]), repository)
    old_expected = payload(source, old_lock["revision"], repository)
    verify_tree(before, old_expected)
    expected = payload(source, source.head, repository)
    try:
        verify_tree(before, expected)
        return None
    except PolicyError:
        pass
    prefix = "/repos/" + repository
    pulls = list(
        api.pages(prefix + "/pulls?state=open&base=" + quote(config["branch"], safe=""))
    )
    candidates = [
        p
        for p in pulls
        if p["head"]["ref"] == SYNC_BRANCH
        and p["head"]["repo"]
        and p["head"]["repo"]["full_name"] == repository
    ]
    if len(candidates) > 1:
        raise PolicyError("Multiple sync PRs need maintainer attention")
    existing = candidates[0] if candidates else None
    ref_path = prefix + "/git/refs/heads/" + quote(SYNC_BRANCH, safe="")
    old_ref = api.request(
        "GET",
        prefix + "/git/ref/heads/" + quote(SYNC_BRANCH, safe=""),
        allow_missing=True,
    )
    if old_ref:
        # A stale sync branch may trail main. Check its actual parent, not main,
        # so unrelated later code commits are preserved when rebuilding the PR.
        owner_pr = existing
        if owner_pr is None:
            closed = api.pages(
                prefix
                + "/pulls?state=closed&head="
                + quote(repository.split("/")[0] + ":" + SYNC_BRANCH, safe="")
            )
            owner_pr = next(
                (
                    p
                    for p in closed
                    if p.get("merged_at")
                    and p["head"]["sha"] == old_ref["object"]["sha"]
                    and p["user"]["id"] == bot_id
                ),
                None,
            )
        if not owner_pr or owner_pr["user"]["id"] != bot_id:
            raise PolicyError(
                "Existing sync branch is not an active or previously merged App PR"
            )
        commit = api.request("GET", prefix + "/git/commits/" + old_ref["object"]["sha"])
        if len(commit["parents"]) not in {1, 2}:
            raise PolicyError("Sync branch has unexpected history")
        parent_sha = commit["parents"][-1]["sha"]
        if not api.is_ancestor(repository, parent_sha, base_sha):
            raise PolicyError(
                "Sync baseline is outside the enrolled branch; refusing to overwrite it"
            )
        parent = api.tree(repository, parent_sha)
        branch_tree = api.tree(repository, old_ref["object"]["sha"])
        branch_lock = read_lock(api.blob(repository, branch_tree[LOCK][1]), repository)
        branch_expected = payload(source, branch_lock["revision"], repository)
        verify_tree(branch_tree, branch_expected)
        managed = set(branch_expected) | set(expected) | set(old_expected)
        if differences(parent, branch_tree) - managed:
            raise PolicyError(
                "Sync branch contains unrelated changes; refusing to overwrite them"
            )
        try:
            verify_tree(branch_tree, expected)
        except PolicyError:
            pass
        else:
            if existing and not differences(before, branch_tree) - managed:
                if not check_pull(
                    api, source, repository, existing["number"], bot_id, merge=True
                ):
                    raise OperationError(
                        "Sync PR was rejected; inspect its failed check"
                    )
                return existing["number"]
    entries = []
    for path in sorted(set(old_expected) | set(expected)):
        data = expected.get(path)
        if data is None:
            entries.append(
                {"path": path, "mode": "100644", "type": "blob", "sha": None}
            )
        else:
            entries.append(
                {
                    "path": path,
                    "mode": "100644",
                    "type": "blob",
                    "content": data.decode(),
                }
            )
    base_commit = api.request("GET", prefix + "/git/commits/" + base_sha)
    tree = api.request(
        "POST",
        prefix + "/git/trees",
        {"base_tree": base_commit["tree"]["sha"], "tree": entries},
    )
    commit = api.request(
        "POST",
        prefix + "/git/commits",
        {
            "message": "chore(standards): sync approved development rules",
            "tree": tree["sha"],
            "parents": [old_ref["object"]["sha"], base_sha] if old_ref else [base_sha],
        },
    )
    # Fast-forward only: a concurrent push makes this update fail instead of
    # overwriting it. Squash merging keeps the protected main history linear.
    if old_ref:
        current = api.request(
            "GET", prefix + "/git/ref/heads/" + quote(SYNC_BRANCH, safe="")
        )
        if current["object"]["sha"] != old_ref["object"]["sha"]:
            raise PolicyError(
                "Sync branch changed during preparation; refusing to overwrite it"
            )
        api.request("PATCH", ref_path, {"sha": commit["sha"], "force": False})
    else:
        api.request(
            "POST",
            prefix + "/git/refs",
            {"ref": "refs/heads/" + SYNC_BRANCH, "sha": commit["sha"]},
        )
    if existing:
        number = existing["number"]
    else:
        pr = api.request(
            "POST",
            prefix + "/pulls",
            {
                "title": "chore(standards): sync approved development rules",
                "head": SYNC_BRANCH,
                "base": config["branch"],
                "body": "Problem:\n- Shared rules are behind the approved central revision.\n\nFix:\n- Synchronize instructions, skills and checks from [the approved source](https://github.com/%s/commit/%s).\n"
                % (SOURCE, source.head),
            },
        )
        number = pr["number"]
    if not check_pull(api, source, repository, number, bot_id, merge=True):
        raise OperationError("Sync PR was rejected; inspect its failed check")
    return number


def reconcile(api, source, bot_id, sync=False, repository=None):
    failures = []
    repositories = registry(source, source.head)
    if repository is not None and repository not in repositories:
        raise PolicyError("Repository is not enrolled")
    for repository in [repository] if repository is not None else repositories:
        if sync:
            try:
                sync_repository(api, source, repository, bot_id)
            except PolicyError as error:
                failures.append(repository + ": " + str(error))
        try:
            branch = registry(source, source.head)[repository]["branch"]
            pulls = api.pages(
                "/repos/%s/pulls?state=open&base=%s"
                % (repository, quote(branch, safe=""))
            )
            for pr in pulls:
                try:
                    accepted = check_pull(
                        api, source, repository, pr["number"], bot_id, merge=True
                    )
                    if not accepted and pr["user"]["id"] == bot_id:
                        failures.append(
                            repository + ": App PR failed policy validation"
                        )
                except PolicyError as error:
                    failures.append(repository + ": " + str(error))
        except PolicyError as error:
            failures.append(repository + ": " + str(error))
    return failures
