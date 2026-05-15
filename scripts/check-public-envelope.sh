#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PUBLIC_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

cd "$PUBLIC_ROOT"

python3 - <<'PY'
import json
import re
import tarfile
from pathlib import Path

root = Path(".").resolve()
beta_tag = "v0.1.1-beta.1"
beta_asset = "cyrune-free-v0.1.1-beta.1.tar.gz"
beta_root = beta_asset.removesuffix(".tar.gz")
alpha_tag = "v0.1.0"


def fail(message: str) -> None:
    raise SystemExit(message)


def read(path: str) -> str:
    return (root / path).read_text(encoding="utf-8")


prepare = read("scripts/prepare-public-run.sh")
required_prepare_fragments = [
    f"/releases/download/{beta_tag}/{beta_asset}",
    f'CARRIER_FILENAME="{beta_asset}"',
    'CARRIER_SIZE_BYTES="',
    'CARRIER_SHA256="',
]
for fragment in required_prepare_fragments:
    if fragment not in prepare:
        fail(f"prepare-public-run.sh missing beta carrier fragment: {fragment}")
if f"/releases/download/{alpha_tag}/" in prepare:
    fail("prepare-public-run.sh still points at the alpha carrier")

docs_required = {
    "README.md": ["public beta", beta_tag, "public alpha snapshot", "OS-level sandbox enforcement"],
    "README.ja.md": ["public beta", beta_tag, "public alpha"],
    "docs/CYRUNE_Free_Public_Index.md": ["Public Beta Claim Boundary", beta_tag],
    "docs/BETA_CRITERIA.md": [beta_tag, beta_asset, "Closed Gate Report"],
    "docs/GETTING_STARTED.md": ["public beta first-success path", "pinned beta carrier"],
    "docs/FIRST_SUCCESS_EXPECTED.md": ["public beta release contract", "production maturity"],
    "docs/USER_GUIDE.md": ["single-user public beta package", "docs/BETA_CRITERIA.md"],
    "docs/ENGINEERING_SPEC.md": ["public beta", "beta release-contract pins"],
    "docs/ja/BETA_CRITERIA.md": [beta_tag, beta_asset],
}
for path, fragments in docs_required.items():
    content = read(path)
    for fragment in fragments:
        if fragment not in content:
            fail(f"{path} missing required beta contract fragment: {fragment}")

for path in [
    "README.md",
    "docs/BETA_CRITERIA.md",
    "docs/CYRUNE_Free_Public_Index.md",
    "docs/GETTING_STARTED.md",
    "docs/FIRST_SUCCESS_EXPECTED.md",
    "docs/USER_GUIDE.md",
    "docs/ENGINEERING_SPEC.md",
]:
    content = read(path)
    if "enforcement-complete classification / MAC" not in content:
        fail(f"{path} lost classification/MAC non-claim boundary")
    if "OS-level sandbox" not in content:
        fail(f"{path} lost OS-level sandbox non-claim boundary")

stage = read("scripts/stage_shipping_readiness.py")
for fragment in [
    'VERSION = "0.1.1-beta.1"',
    'ARCHIVE_BASENAME = "cyrune-free-v0.1.1-beta.1"',
    'RELEASE_PREPARATION_METADATA_VERSION = "public-beta-release-contract.v1"',
]:
    if fragment not in stage:
        fail(f"stage_shipping_readiness.py missing beta fragment: {fragment}")

publish = read("scripts/publish_release_package_to_github.py")
for fragment in [
    'RELEASE_TAG = "v0.1.1-beta.1"',
    'ASSET_FILENAME = "cyrune-free-v0.1.1-beta.1.tar.gz"',
    '"prerelease": True',
    "existing release tag must not move",
]:
    if fragment not in publish:
        fail(f"publish_release_package_to_github.py missing beta fragment: {fragment}")

archive = root / "target/public-run" / beta_asset
if archive.exists():
    with tarfile.open(archive, "r:gz") as handle:
        member_name = f"{beta_root}/RELEASE_MANIFEST.json"
        try:
            member = handle.getmember(member_name)
        except KeyError as exc:
            raise SystemExit(f"carrier archive missing {member_name}") from exc
        extracted = handle.extractfile(member)
        if extracted is None:
            fail("carrier manifest extraction failed")
        manifest = json.loads(extracted.read().decode("utf-8"))
    expected_pairs = {
        "version": "0.1.1-beta.1",
        "distribution_unit": beta_asset,
        "package_root": beta_root,
        "integrity_mode": "sha256",
        "update_policy": "fixed-distribution/no-self-update",
    }
    for key, expected in expected_pairs.items():
        actual = manifest.get(key)
        if actual != expected:
            fail(f"carrier manifest {key} mismatch: {actual!r} != {expected!r}")

markdown_paths = [
    path
    for path in root.rglob("*.md")
    if "target" not in path.parts and ".git" not in path.parts
]
link_pattern = re.compile(r"\[[^\]]+\]\(([^)]+)\)")
for path in markdown_paths:
    text = path.read_text(encoding="utf-8")
    for match in link_pattern.finditer(text):
        target = match.group(1).split("#", 1)[0]
        if not target or re.match(r"^[a-zA-Z][a-zA-Z0-9+.-]*:", target):
            continue
        if target.startswith("<") and target.endswith(">"):
            target = target[1:-1]
        candidate = (path.parent / target).resolve()
        try:
            candidate.relative_to(root)
        except ValueError:
            fail(f"{path.relative_to(root)} has link outside repository: {target}")
        if not candidate.exists():
            fail(f"{path.relative_to(root)} has missing relative link: {target}")

print("public envelope static checks passed")
PY
