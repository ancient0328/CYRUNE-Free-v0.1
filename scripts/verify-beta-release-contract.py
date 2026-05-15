#!/usr/bin/env python3
"""Fail-closed CYRUNE public beta release-contract verifier."""

from __future__ import annotations

import datetime as _dt
import hashlib
import json
import os
from pathlib import Path
import subprocess
import sys
import urllib.error
import urllib.request


SCHEMA_VERSION = "cyrune.free.beta-release-contract-report.v1"
C5_SCHEMA_VERSION = "cyrune.free.first-success-verifier-report.v1"
REPOSITORY = "ancient0328/CYRUNE"
SOURCE_SHA = "062cd58548e9f66e2371f580edae8f641d0d05f7"
TAG = "v0.1.1-beta.1"
TAG_TARGET = "61eb4c68630600d9b1a7f325fd6d06759ede846c"
RELEASE_ID = 313683966
ASSET_ID = 405673798
ASSET_NAME = "cyrune-free-v0.1.1-beta.1.tar.gz"
ASSET_SIZE = 563982199
ASSET_DIGEST = "sha256:73654922f0f1c170ce34001d6f1021b72ec9eb8c28aa8a81a3d572ccde00c938"
POLICY_PACK_ID = "cyrune-free-default"
PUBLIC_FIRST_SUCCESS_INPUT = "ship-goal public first success"
RUN_MODE = "no_llm"
FIRST_SUCCESS_REPORT = "target/public-run/first-success-report.json"
CGR_OUTPUT_TARGET = (
    "../dev-docs/90-reports/"
    "20260430-public-EVID-BETA-2-terminal-evidence-release-contract.md"
)

REQUIRED_ARGS = {
    "--candidate-root",
    "--source-sha",
    "--tag-target",
    "--release-id",
    "--asset-id",
    "--asset-digest",
    "--ci-run-id",
    "--cgr-output-target",
    "--first-success-report",
}
FORBIDDEN_CGR_PATH_ARG = "--" + "cgr-path"


class BetaFailure(Exception):
    def __init__(self, code: str, message: str, diagnostics: list[str] | None = None) -> None:
        super().__init__(message)
        self.code = code
        self.message = message
        self.diagnostics = diagnostics or []


class Context:
    def __init__(self) -> None:
        self.candidate_root: Path | None = None
        self.first_success_state_root: str | None = None
        self.first_success_cyrune_home: str | None = None


def checked_at() -> str:
    return _dt.datetime.now(_dt.timezone.utc).isoformat().replace("+00:00", "Z")


def fail(code: str, message: str, diagnostics: list[str] | None = None) -> None:
    raise BetaFailure(code, message, diagnostics)


def emit_failure(failure: BetaFailure, ctx: Context) -> int:
    report = {
        "schema_version": SCHEMA_VERSION,
        "verified": False,
        "failure_code": failure.code,
        "failure_message": failure.message,
        "diagnostics": failure.diagnostics,
        "candidate_root": str(ctx.candidate_root) if ctx.candidate_root else None,
        "first_success_state_root": ctx.first_success_state_root,
        "first_success_cyrune_home": ctx.first_success_cyrune_home,
        "checked_at": checked_at(),
    }
    print(json.dumps(report, indent=2, sort_keys=True))
    print(failure.code, file=sys.stderr)
    return 1


def parse_args(argv: list[str]) -> dict[str, str]:
    if FORBIDDEN_CGR_PATH_ARG in argv:
        fail("BETA-MUTABLE-INPUT", "legacy CGR path argument is forbidden")
    parsed: dict[str, str] = {}
    index = 0
    while index < len(argv):
        key = argv[index]
        if key not in REQUIRED_ARGS:
            fail("BETA-MUTABLE-INPUT", f"unknown or mutable argument: {key}")
        if key in parsed:
            fail("BETA-MUTABLE-INPUT", f"duplicate argument: {key}")
        if index + 1 >= len(argv):
            fail("BETA-MISSING-ARG", f"{key} requires a value")
        value = argv[index + 1]
        if value.startswith("--"):
            fail("BETA-MISSING-ARG", f"{key} requires a value")
        parsed[key] = value
        index += 2
    missing = sorted(REQUIRED_ARGS - set(parsed))
    if missing:
        fail("BETA-MISSING-ARG", f"missing required argument(s): {', '.join(missing)}")
    return parsed


def reject_mutable(value: str, name: str) -> None:
    if value == "" or value in {"latest", "main", "master", "HEAD"}:
        fail("BETA-MUTABLE-INPUT", f"{name} is mutable or empty: {value!r}")


def require_exact(args: dict[str, str], key: str, expected: str, code: str) -> str:
    value = args[key]
    reject_mutable(value, key)
    if value != expected:
        fail(code, f"{key} mismatch: expected {expected}, got {value}")
    return value


def require_int(args: dict[str, str], key: str, expected: int | None, code: str) -> int:
    value = args[key]
    reject_mutable(value, key)
    try:
        parsed = int(value)
    except ValueError:
        fail("BETA-MUTABLE-INPUT", f"{key} is not an integer: {value!r}")
    if expected is not None and parsed != expected:
        fail(code, f"{key} mismatch: expected {expected}, got {parsed}")
    return parsed


def resolve_candidate_root(value: str, ctx: Context) -> Path:
    reject_mutable(value, "--candidate-root")
    candidate = Path(value)
    if not candidate.is_absolute():
        fail("BETA-CANDIDATE-ROOT-INVALID", "--candidate-root must be absolute")
    try:
        resolved = candidate.resolve(strict=True)
    except OSError as error:
        fail("BETA-CANDIDATE-ROOT-INVALID", f"candidate root does not resolve: {error}")
    if not resolved.is_dir():
        fail("BETA-CANDIDATE-ROOT-INVALID", "candidate root is not a directory")
    ctx.candidate_root = resolved
    for relative in [
        "README.md",
        "docs/BETA_CRITERIA.md",
        "scripts/first-success.sh",
        "scripts/check-beta-release-contract.sh",
        ".github/workflows/public-ci.yml",
        "Cargo.toml",
    ]:
        if not (resolved / relative).exists():
            fail("BETA-CANDIDATE-ROOT-INVALID", f"candidate root missing {relative}")
    return resolved


def resolve_relative_under(root: Path, value: str, code: str) -> Path:
    path = Path(value)
    if path.is_absolute() or any(part == ".." for part in path.parts):
        fail(code, f"path must be candidate-root relative and cannot contain '..': {value}")
    resolved = (root / path).resolve(strict=False)
    try:
        resolved.relative_to(root)
    except ValueError:
        fail(code, f"path escapes candidate root: {value}")
    return resolved


def resolve_cgr_target(root: Path, value: str) -> Path:
    if value != CGR_OUTPUT_TARGET:
        fail("BETA-CGR-TARGET-MISMATCH", f"--cgr-output-target must be {CGR_OUTPUT_TARGET}")
    path = Path(value)
    resolved = (root / path).resolve(strict=False)
    expected_parent = (root.parent / "dev-docs" / "90-reports").resolve(strict=False)
    if resolved.parent != expected_parent:
        fail("BETA-CGR-TARGET-MISMATCH", f"CGR target parent mismatch: {resolved.parent}")
    return resolved


def resolve_report_paths(root: Path, args: dict[str, str]) -> tuple[Path, Path]:
    if args["--first-success-report"] != FIRST_SUCCESS_REPORT:
        fail(
            "BETA-FIRST-SUCCESS-REPORT-INVALID",
            f"--first-success-report must be {FIRST_SUCCESS_REPORT}",
        )
    first_success = resolve_relative_under(
        root, args["--first-success-report"], "BETA-CANDIDATE-ROOT-INVALID"
    )
    cgr_target = resolve_cgr_target(root, args["--cgr-output-target"])
    if not first_success.is_file():
        fail("BETA-FIRST-SUCCESS-REPORT-MISSING", f"missing first-success report: {first_success}")
    return first_success, cgr_target


def github_json(path: str) -> dict:
    url = f"https://api.github.com/repos/{REPOSITORY}/{path}"
    headers = {
        "Accept": "application/vnd.github+json",
        "User-Agent": "cyrune-public-beta-release-verifier",
    }
    token = os.environ.get("GITHUB_TOKEN")
    if token:
        headers["Authorization"] = f"Bearer {token}"
    request = urllib.request.Request(url, headers=headers)
    try:
        with urllib.request.urlopen(request, timeout=30) as response:
            return json.loads(response.read().decode("utf-8"))
    except (urllib.error.URLError, urllib.error.HTTPError, TimeoutError, json.JSONDecodeError) as error:
        fail("BETA-GITHUB-API", f"GitHub API read failed for {path}: {error}")


def check_github_surfaces(args: dict[str, str], ci_run_id: int) -> dict:
    main_ref = github_json("git/ref/heads/main")
    if main_ref.get("object", {}).get("sha") != args["--source-sha"]:
        fail("BETA-SOURCE-SHA-MISMATCH", "remote main SHA mismatch")

    tag_ref = github_json(f"git/ref/tags/{TAG}")
    if tag_ref.get("object", {}).get("sha") != args["--tag-target"]:
        fail("BETA-TAG-TARGET-MISMATCH", "remote tag target mismatch")

    release = github_json(f"releases/tags/{TAG}")
    if release.get("id") != RELEASE_ID or release.get("tag_name") != TAG:
        fail("BETA-RELEASE-ID-MISMATCH", "release id or tag mismatch")
    if release.get("target_commitish") != args["--tag-target"]:
        fail("BETA-TAG-TARGET-MISMATCH", "release target_commitish mismatch")
    if release.get("prerelease") is not True:
        fail("BETA-RELEASE-ID-MISMATCH", "release is not marked prerelease")
    assets = release.get("assets")
    if not isinstance(assets, list):
        fail("BETA-ASSET-ID-MISMATCH", "release assets are missing")
    asset = next((item for item in assets if item.get("id") == ASSET_ID), None)
    if not asset or asset.get("name") != ASSET_NAME:
        fail("BETA-ASSET-ID-MISMATCH", "release asset id/name mismatch")
    if asset.get("size") != ASSET_SIZE:
        fail("BETA-ASSET-SIZE-MISMATCH", "release asset size mismatch")
    if asset.get("digest") != args["--asset-digest"]:
        fail("BETA-ASSET-DIGEST-MISMATCH", "release asset digest mismatch")

    run = github_json(f"actions/runs/{ci_run_id}")
    if run.get("name") != "public-ci" or run.get("head_sha") not in {
        args["--source-sha"],
        args["--tag-target"],
    }:
        fail("BETA-CI-RUN-MISMATCH", "CI run is not the expected public-ci source/tag run")
    if run.get("status") != "completed" or run.get("conclusion") != "success":
        fail("BETA-CI-NOT-SUCCESS", "CI run is not completed/success")
    return {"release": release, "asset": asset, "ci": run}


def check_local_checkout(root: Path, source_sha: str, tag_target: str) -> str:
    if not (root / ".git").exists():
        return "none"
    try:
        status = subprocess.run(
            ["git", "-C", str(root), "status", "--porcelain"],
            check=False,
            capture_output=True,
            text=True,
        )
    except OSError as error:
        fail("BETA-LOCAL-GIT-INVALID", f"git status failed to start: {error}")
    if status.returncode != 0:
        fail("BETA-LOCAL-GIT-INVALID", "git status failed", [status.stderr.strip()])
    if status.stdout.strip():
        fail("BETA-DIRTY-LOCAL-CHECKOUT", "candidate root local checkout is dirty")
    try:
        head = subprocess.run(
            ["git", "-C", str(root), "rev-parse", "HEAD"],
            check=False,
            capture_output=True,
            text=True,
        )
    except OSError as error:
        fail("BETA-LOCAL-GIT-INVALID", f"git rev-parse HEAD failed to start: {error}")
    if head.returncode != 0:
        fail("BETA-LOCAL-GIT-INVALID", "git rev-parse HEAD failed", [head.stderr.strip()])
    head_sha = head.stdout.strip()
    if head_sha == source_sha:
        return "source"
    if head_sha == tag_target:
        return "tag"
    fail("BETA-STALE-LOCAL-CHECKOUT", f"candidate root HEAD is stale: {head_sha}")


def load_json(path: Path, code: str) -> dict:
    try:
        with path.open("r", encoding="utf-8") as handle:
            data = json.load(handle)
    except OSError as error:
        fail(code, f"failed to read {path}: {error}")
    except json.JSONDecodeError as error:
        fail("BETA-FIRST-SUCCESS-REPORT-INVALID", f"failed to parse {path}: {error}")
    if not isinstance(data, dict):
        fail("BETA-FIRST-SUCCESS-REPORT-INVALID", f"{path} is not a JSON object")
    return data


def sha256_file(path: Path) -> str:
    try:
        digest = hashlib.sha256(path.read_bytes()).hexdigest()
    except OSError as error:
        fail("BETA-FIRST-SUCCESS-ROOT-MISMATCH", f"failed to read {path}: {error}")
    return f"sha256:{digest}"


def require_str(data: dict, key: str, code: str) -> str:
    value = data.get(key)
    if not isinstance(value, str) or value == "":
        fail(code, f"missing required string field: {key}")
    return value


def verify_first_success_report(root: Path, report_path: Path, ctx: Context) -> dict:
    report = load_json(report_path, "BETA-FIRST-SUCCESS-REPORT-MISSING")
    ctx.first_success_state_root = report.get("state_root") if isinstance(report.get("state_root"), str) else None
    ctx.first_success_cyrune_home = report.get("cyrune_home") if isinstance(report.get("cyrune_home"), str) else None
    if report.get("schema_version") != C5_SCHEMA_VERSION:
        fail("BETA-FIRST-SUCCESS-REPORT-INVALID", "first-success report schema_version mismatch")
    if report.get("public_first_success_input") != PUBLIC_FIRST_SUCCESS_INPUT:
        fail("BETA-FIRST-SUCCESS-REPORT-INVALID", "first-success input mismatch")
    if report.get("run_mode") != RUN_MODE:
        fail("BETA-FIRST-SUCCESS-REPORT-INVALID", "first-success run_mode mismatch")
    if report.get("failure_message") is not None:
        fail("BETA-FIRST-SUCCESS-REPORT-INVALID", "first-success failure_message is not null")
    if report.get("verified") is not True or report.get("outcome") != "accepted" or report.get("failure_code") is not None:
        fail("BETA-FIRST-SUCCESS-REPORT-INVALID", "first-success report is not verified accepted")
    response = report.get("response")
    if not isinstance(response, dict):
        fail("BETA-FIRST-SUCCESS-REPORT-INVALID", "first-success response is missing")
    if response.get("outcome") != "accepted":
        fail("BETA-FIRST-SUCCESS-REPORT-INVALID", "first-success response outcome is not accepted")

    for key in ["evidence_id", "correlation_id", "run_id", "policy_pack_id", "citation_bundle_id", "working_hash_after"]:
        if response.get(key) != report.get(key):
            fail("BETA-FIRST-SUCCESS-ID-MISMATCH", f"first-success {key} mismatch")
    if report.get("policy_pack_id") != POLICY_PACK_ID:
        fail("BETA-FIRST-SUCCESS-REPORT-INVALID", "first-success policy mismatch")
    if report.get("working_hash_after") != report.get("working_json_hash"):
        fail("BETA-FIRST-SUCCESS-ID-MISMATCH", "first-success working hash summary mismatch")
    evidence_id = require_str(report, "evidence_id", "BETA-FIRST-SUCCESS-REPORT-INVALID")
    expected_marker = f"ledger/terminal-bindings/{evidence_id}.json"
    if report.get("terminal_binding_path") != expected_marker:
        fail("BETA-FIRST-SUCCESS-ID-MISMATCH", "terminal binding path summary mismatch")

    expected_state_root = (root / "target/public-run").resolve(strict=False)
    expected_home = (expected_state_root / "home").resolve(strict=False)
    if report.get("state_root") != str(expected_state_root):
        fail("BETA-FIRST-SUCCESS-ROOT-MISMATCH", "first-success state_root mismatch")
    if report.get("cyrune_home") != str(expected_home):
        fail("BETA-FIRST-SUCCESS-ROOT-MISMATCH", "first-success cyrune_home mismatch")

    cyrune_home = Path(require_str(report, "cyrune_home", "BETA-FIRST-SUCCESS-REPORT-INVALID"))
    if cyrune_home.resolve(strict=False) != expected_home:
        fail("BETA-FIRST-SUCCESS-ROOT-MISMATCH", "first-success home path does not bind to candidate root")
    terminal_rel = require_str(report, "terminal_binding_path", "BETA-FIRST-SUCCESS-REPORT-INVALID")
    evidence_rel = require_str(report, "evidence_dir", "BETA-FIRST-SUCCESS-REPORT-INVALID")
    terminal_path = resolve_relative_under(cyrune_home, terminal_rel, "BETA-FIRST-SUCCESS-ROOT-MISMATCH")
    evidence_dir = resolve_relative_under(cyrune_home, evidence_rel, "BETA-FIRST-SUCCESS-ROOT-MISMATCH")
    if not terminal_path.is_file():
        fail("BETA-FIRST-SUCCESS-ROOT-MISMATCH", "terminal binding file is missing")
    if not evidence_dir.is_dir():
        fail("BETA-FIRST-SUCCESS-ROOT-MISMATCH", "evidence directory is missing")
    if sha256_file(cyrune_home / "working" / "working.json") != report.get("working_json_hash"):
        fail("BETA-FIRST-SUCCESS-ROOT-MISMATCH", "working raw hash mismatch")
    if sha256_file(evidence_dir / "manifest.json") != report.get("evidence_manifest_hash"):
        fail("BETA-FIRST-SUCCESS-ROOT-MISMATCH", "manifest raw hash mismatch")
    if sha256_file(evidence_dir / "hashes.json") != report.get("evidence_hashes_hash"):
        fail("BETA-FIRST-SUCCESS-ROOT-MISMATCH", "hashes raw hash mismatch")
    return report


def success_report(
    args: dict[str, str],
    root: Path,
    local_checkout_line: str,
    ci_run_id: int,
    first_success: dict,
    cgr_target: str,
    diagnostics: list[str],
) -> dict:
    return {
        "schema_version": SCHEMA_VERSION,
        "verified": True,
        "failure_code": None,
        "failure_message": None,
        "repository": REPOSITORY,
        "candidate_root": str(root),
        "local_checkout_line": local_checkout_line,
        "source_sha": args["--source-sha"],
        "tag": TAG,
        "tag_target": args["--tag-target"],
        "release_id": int(args["--release-id"]),
        "asset_id": int(args["--asset-id"]),
        "asset_name": ASSET_NAME,
        "asset_size": ASSET_SIZE,
        "asset_digest": args["--asset-digest"],
        "ci_run_id": ci_run_id,
        "ci_conclusion": "success",
        "first_success_report": first_success,
        "first_success_evidence_id": first_success.get("evidence_id"),
        "first_success_run_id": first_success.get("run_id"),
        "first_success_correlation_id": first_success.get("correlation_id"),
        "first_success_state_root": first_success.get("state_root"),
        "first_success_cyrune_home": first_success.get("cyrune_home"),
        "first_success_root_binding": "candidate_root",
        "terminal_binding_path": first_success.get("terminal_binding_path"),
        "cgr_output_target": cgr_target,
        "checked_at": checked_at(),
        "diagnostics": diagnostics,
    }


def verify(argv: list[str], ctx: Context) -> int:
    args = parse_args(argv)
    source_sha = require_exact(args, "--source-sha", SOURCE_SHA, "BETA-SOURCE-SHA-MISMATCH")
    tag_target = require_exact(args, "--tag-target", TAG_TARGET, "BETA-TAG-TARGET-MISMATCH")
    require_int(args, "--release-id", RELEASE_ID, "BETA-RELEASE-ID-MISMATCH")
    require_int(args, "--asset-id", ASSET_ID, "BETA-ASSET-ID-MISMATCH")
    require_exact(args, "--asset-digest", ASSET_DIGEST, "BETA-ASSET-DIGEST-MISMATCH")
    ci_run_id = require_int(args, "--ci-run-id", None, "BETA-CI-RUN-MISMATCH")
    root = resolve_candidate_root(args["--candidate-root"], ctx)
    first_success_path, _ = resolve_report_paths(root, args)
    local_checkout_line = check_local_checkout(root, source_sha, tag_target)
    check_github_surfaces(args, ci_run_id)
    first_success = verify_first_success_report(root, first_success_path, ctx)
    report = success_report(
        args,
        root,
        local_checkout_line,
        ci_run_id,
        first_success,
        args["--cgr-output-target"],
        [],
    )
    print(json.dumps(report, indent=2, sort_keys=True))
    return 0


def main(argv: list[str]) -> int:
    ctx = Context()
    try:
        return verify(argv, ctx)
    except BetaFailure as failure:
        return emit_failure(failure, ctx)


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:]))
