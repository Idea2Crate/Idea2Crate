"""Small bounded GitHub client; credentials never appear in diagnostics."""

import base64
import json
import os
from urllib.error import HTTPError, URLError
from urllib.parse import quote
from urllib.request import HTTPRedirectHandler, Request, build_opener

from .bundle import CHECK, OperationError


class NoRedirect(HTTPRedirectHandler):
    def redirect_request(self, req, fp, code, msg, headers, newurl):
        raise OperationError(
            "Refusing to forward App credentials through an API redirect"
        )


class GitHub:
    def __init__(self, token=None, api="https://api.github.com"):
        self.token = token or os.environ.get("GH_TOKEN")
        if not self.token:
            raise OperationError(
                "GH_TOKEN must contain a scoped GitHub App installation token"
            )
        if api != "https://api.github.com":
            raise OperationError("Only the GitHub.com API is supported")
        self.api = api

    def request(self, method, path, data=None, allow_missing=False):
        if not path.startswith("/") or path.startswith("//"):
            raise OperationError("Invalid API path")
        request = Request(
            self.api + path,
            data=None if data is None else json.dumps(data).encode(),
            method=method,
            headers={
                "Authorization": "Bearer " + self.token,
                "Accept": "application/vnd.github+json",
                "X-GitHub-Api-Version": "2022-11-28",
                "Content-Type": "application/json",
                "User-Agent": "Idea2Crate-standards",
            },
        )
        try:
            with build_opener(NoRedirect()).open(request, timeout=30) as response:
                # Bound responses before parsing; never print bodies on failure.
                raw = response.read(16 * 1024 * 1024 + 1)
                if len(raw) > 16 * 1024 * 1024:
                    raise OperationError("GitHub response exceeds the supported limit")
                return json.loads(raw) if raw else None
        except HTTPError as exc:
            if allow_missing and exc.code == 404:
                return None
            raise OperationError(
                "GitHub %s %s failed (HTTP %s)" % (method, path, exc.code)
            ) from None
        except (URLError, TimeoutError, ValueError) as exc:
            raise OperationError("GitHub request could not be verified") from exc

    def pages(self, path):
        separator = "&" if "?" in path else "?"
        for page in range(1, 102):
            if page > 100:
                raise OperationError("Pagination exceeded the supported limit")
            values = self.request(
                "GET", path + separator + "per_page=100&page=" + str(page)
            )
            if not isinstance(values, list):
                raise OperationError("Invalid paginated GitHub response")
            yield from values
            if len(values) < 100:
                return

    def tree(self, repository, revision):
        value = self.request(
            "GET", "/repos/%s/git/trees/%s?recursive=1" % (repository, revision)
        )
        if value.get("truncated") or not isinstance(value.get("tree"), list):
            raise OperationError("Cannot validate a truncated Git tree")
        return {
            item["path"]: (item["mode"], item["sha"])
            for item in value["tree"]
            if item["type"] != "tree"
        }

    def blob(self, repository, sha):
        value = self.request("GET", "/repos/%s/git/blobs/%s" % (repository, sha))
        if value.get("encoding") != "base64":
            raise OperationError("Unsupported GitHub blob encoding")
        try:
            return base64.b64decode(value["content"].replace("\n", ""), validate=True)
        except (ValueError, KeyError) as exc:
            raise OperationError("Invalid GitHub blob") from exc

    def pull(self, repository, number):
        return self.request("GET", "/repos/%s/pulls/%s" % (repository, number))

    def branch(self, repository, name):
        return self.request(
            "GET", "/repos/%s/git/ref/heads/%s" % (repository, quote(name, safe=""))
        )

    def is_ancestor(self, repository, ancestor, descendant):
        value = self.request(
            "GET", "/repos/%s/compare/%s...%s" % (repository, ancestor, descendant)
        )
        status = value.get("status")
        if status not in {"identical", "ahead", "behind", "diverged"}:
            raise OperationError("Cannot verify sync branch ancestry")
        return status in {"identical", "ahead"}

    def check(self, repository, sha, conclusion, summary):
        existing = self.request(
            "GET",
            "/repos/%s/commits/%s/check-runs?check_name=%s&per_page=100"
            % (repository, sha, quote(CHECK, safe="")),
        )
        app_id = int(os.environ["STANDARDS_APP_ID"])
        for check in existing.get("check_runs", []):
            if (
                check.get("app", {}).get("id") == app_id
                and check.get("conclusion") == conclusion
                and check.get("output", {}).get("summary") == summary
            ):
                return check
        return self.request(
            "POST",
            "/repos/%s/check-runs" % repository,
            {
                "name": CHECK,
                "head_sha": sha,
                "status": "completed",
                "conclusion": conclusion,
                "output": {"title": "Shared development standards", "summary": summary},
            },
        )

    def enable_auto_merge(self, node_id):
        # GraphQL has no expected-head argument. Verify the expected SHA before
        # calling; the independent check must pass again on every later commit.
        value = self.request(
            "POST",
            "/graphql",
            {
                "query": "mutation($id:ID!){enablePullRequestAutoMerge(input:{pullRequestId:$id,mergeMethod:SQUASH}){pullRequest{autoMergeRequest{enabledAt}}}}",
                "variables": {"id": node_id},
            },
        )
        if value.get("errors"):
            raise OperationError(
                "GitHub could not enable auto-merge; check repository settings"
            )
