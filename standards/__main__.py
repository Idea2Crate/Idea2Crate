"""CLI for reviewable plans, local checks and trusted central automation."""

import argparse
import json
import os
import sys
from pathlib import Path

from .automation import check_pull, reconcile
from .bundle import GitSource, PolicyError, digest, local_check, payload
from .github import GitHub
from .install import apply_local


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "command", choices=["plan", "apply", "check-local", "check-pr", "reconcile"]
    )
    parser.add_argument("--source", default=str(Path(__file__).resolve().parents[1]))
    parser.add_argument("--repository")
    parser.add_argument("--revision")
    parser.add_argument("--target", default=".")
    parser.add_argument("--pull", type=int)
    parser.add_argument("--sync", action="store_true")
    args = parser.parse_args()
    source = GitSource(args.source)
    if args.command == "check-local":
        local_check(source, args.target)
        return 0
    if args.command == "apply":
        if not args.repository:
            parser.error("apply requires --repository")
        changed = apply_local(
            source, args.target, args.repository, args.revision or source.head
        )
        print(json.dumps({"changed": changed}, indent=2))
        return 0
    if args.command == "plan":
        if not args.repository:
            parser.error("plan requires --repository")
        files = payload(source, args.revision or source.head, args.repository)
        print(
            json.dumps(
                {
                    p: {"bytes": len(data), "sha256": digest(data)}
                    for p, data in files.items()
                },
                indent=2,
            )
        )
        return 0
    if (
        os.environ.get("GITHUB_REPOSITORY") != "Idea2Crate/standards"
        or os.environ.get("GITHUB_REF") != "refs/heads/main"
    ):
        raise PolicyError(
            "Credentialed automation must run on central main, never PR code"
        )
    if os.environ.get("GITHUB_SHA") != source.head:
        raise PolicyError("Checkout does not match the trusted central run")
    bot_id = int(os.environ["STANDARDS_BOT_ID"])
    api = GitHub()
    if args.command == "check-pr":
        if not args.repository or not args.pull:
            parser.error("check-pr requires --repository and --pull")
        return int(not check_pull(api, source, args.repository, args.pull, bot_id))
    failures = reconcile(api, source, bot_id, sync=args.sync)
    for error in failures:
        print(error, file=sys.stderr)
    return int(bool(failures))


if __name__ == "__main__":
    try:
        sys.exit(main())
    except (PolicyError, OSError, ValueError, KeyError) as error:
        print("Standards automation failed: " + str(error), file=sys.stderr)
        sys.exit(1)
