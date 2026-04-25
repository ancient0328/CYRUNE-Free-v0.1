#!/usr/bin/env python3
from __future__ import annotations

import argparse
import base64
import hashlib
import json
import ssl
import subprocess
import tarfile
import tempfile
import time
from dataclasses import dataclass
from pathlib import Path
from typing import Any
from urllib.error import HTTPError, URLError
from urllib.request import Request, urlopen

from prepare_github_publication_branch import (
    materialize_tracked_publication_branch,
    resolve_roots as resolve_publication_branch_roots,
)

try:
    import certifi
except ImportError:  # pragma: no cover - environment-dependent fallback
    certifi = None


REPOSITORY_OWNER = "ancient0328"
REPOSITORY_NAME = "CYRUNE"
REPOSITORY_FULL_NAME = f"{REPOSITORY_OWNER}/{REPOSITORY_NAME}"
BRANCH_NAME = "main"
BRANCH_REF = f"refs/heads/{BRANCH_NAME}"
RELEASE_TAG = "v0.1.0"
COMPATIBILITY_RELEASE_TAGS = {"v0.1"}
RELEASE_TITLE = "CYRUNE Free v0.1 public alpha"
RELEASE_BODY = ""
ASSET_FILENAME = "cyrune-free-v0.1.tar.gz"
RELEASE_LANDING_PAGE = (
    f"https://github.com/{REPOSITORY_FULL_NAME}/releases/tag/{RELEASE_TAG}"
)
EXACT_ASSET_URL = (
    f"https://github.com/{REPOSITORY_FULL_NAME}/releases/download/"
    f"{RELEASE_TAG}/{ASSET_FILENAME}"
)
REPOSITORY_ROOT_PAGE = f"https://github.com/{REPOSITORY_FULL_NAME}"
CLONE_URL = f"https://github.com/{REPOSITORY_FULL_NAME}.git"
COMMIT_MESSAGE = "Publish GitHub release package surface"
BOOTSTRAP_MESSAGE = "Bootstrap public release package repository root"
URL_RETRY_COUNT = 5
URL_RETRY_DELAY_SECONDS = 2.0
ROLLBACK_RETRY_COUNT = 5
ROLLBACK_RETRY_DELAY_SECONDS = 2.0


class CommandFailure(RuntimeError):
    pass


def fail(message: str) -> RuntimeError:
    return RuntimeError(message)


def build_ssl_context() -> ssl.SSLContext:
    if certifi is not None:
        return ssl.create_default_context(cafile=certifi.where())
    return ssl.create_default_context()


@dataclass(frozen=True)
class Roots:
    script_path: Path
    script_dir: Path
    standalone_root: Path
    version_root: Path
    free_family_root: Path
    source_root: Path
    public_root: Path
    public_branch_root: Path
    package_asset: Path


@dataclass(frozen=True)
class LocalBlob:
    path: str
    mode: str
    blob_sha: str
    data: bytes


@dataclass(frozen=True)
class RemoteBlob:
    path: str
    mode: str
    blob_sha: str


@dataclass(frozen=True)
class ReleaseAsset:
    asset_id: int
    name: str
    size: int
    download_url: str


@dataclass(frozen=True)
class ReleaseBaseline:
    main_commit_sha: str
    tag_sha: str | None
    release_exists: bool
    rollback_asset_path: Path | None


def run(
    cmd: list[str],
    *,
    input_text: str | None = None,
    check: bool = True,
) -> subprocess.CompletedProcess[str]:
    completed = subprocess.run(
        cmd,
        input=input_text,
        text=True,
        capture_output=True,
        check=False,
    )
    if check and completed.returncode != 0:
        raise CommandFailure(
            f"command failed: {' '.join(cmd)}\nstdout:\n{completed.stdout}\nstderr:\n{completed.stderr}"
        )
    return completed


def gh_api(
    endpoint: str,
    *,
    method: str = "GET",
    payload: dict[str, Any] | None = None,
    allow_404: bool = False,
) -> Any:
    cmd = [
        "gh",
        "api",
        endpoint,
        "--method",
        method,
        "-H",
        "Accept: application/vnd.github+json",
    ]
    input_text: str | None = None
    if payload is not None:
        cmd.extend(["--input", "-"])
        input_text = json.dumps(payload)

    completed = run(cmd, input_text=input_text, check=False)
    if completed.returncode != 0:
        if allow_404 and "HTTP 404" in completed.stderr:
            return None
        raise CommandFailure(
            f"gh api failed: {' '.join(cmd)}\nstdout:\n{completed.stdout}\nstderr:\n{completed.stderr}"
        )

    stdout = completed.stdout.strip()
    if not stdout:
        return None
    return json.loads(stdout)


def resolve_roots(
    *,
    require_publication_roots: bool = True,
    require_package_asset: bool = True,
) -> Roots:
    script_path = Path(__file__).resolve()
    script_dir = script_path.parent
    standalone_root = script_dir.parent
    version_root = standalone_root.parent
    free_family_root = version_root.parent
    source_root = free_family_root.parent
    public_root = source_root / "public" / "free-v0.1"
    public_branch_root = public_root / "free" / "v0.1" / "0"

    if script_dir != standalone_root / "scripts":
        raise fail(f"unexpected SCRIPT_DIR: {script_dir}")
    if standalone_root.name != "0":
        raise fail(f"unexpected STANDALONE_ROOT: {standalone_root}")
    if version_root.name != "v0.1":
        raise fail(f"unexpected VERSION_ROOT: {version_root}")
    if free_family_root.name != "free":
        raise fail(f"unexpected FREE_FAMILY_ROOT: {free_family_root}")
    if source_root.name != "CYRUNE" or source_root.parent.name != "Distro":
        raise fail(f"unexpected SOURCE_ROOT: {source_root}")

    package_asset = standalone_root / "target" / "shipping" / "S2" / ASSET_FILENAME

    if require_package_asset:
        for path in (package_asset,):
            if not path.exists():
                raise fail(f"missing required path: {path}")
            if path.is_symlink():
                raise fail(f"symlink is forbidden: {path}")
    if require_publication_roots:
        for path in (public_root, public_branch_root):
            if not path.exists():
                raise fail(f"missing required path: {path}")
            if path.is_symlink():
                raise fail(f"symlink is forbidden: {path}")
    if require_package_asset and not package_asset.is_file():
        raise fail(f"required path must be a regular file: {package_asset}")
    if require_publication_roots:
        if not public_root.is_dir():
            raise fail(f"required path must be a directory: {public_root}")
        if not public_branch_root.is_dir():
            raise fail(f"required path must be a directory: {public_branch_root}")

    if package_asset.name != ASSET_FILENAME:
        raise fail(f"unexpected package asset filename: {package_asset.name}")

    return Roots(
        script_path=script_path,
        script_dir=script_dir,
        standalone_root=standalone_root,
        version_root=version_root,
        free_family_root=free_family_root,
        source_root=source_root,
        public_root=public_root,
        public_branch_root=public_branch_root,
        package_asset=package_asset,
    )


def ensure_host_prerequisite() -> None:
    run(["gh", "--version"])
    run(["gh", "auth", "status"])
    login = run(["gh", "api", "user", "--jq", ".login"]).stdout.strip()
    if login != REPOSITORY_OWNER:
        raise fail(f"unexpected gh login: {login}")


def git_blob_sha(data: bytes) -> str:
    header = f"blob {len(data)}\0".encode("utf-8")
    return hashlib.sha1(header + data).hexdigest()


def load_local_tracked_blobs(roots: Roots) -> tuple[Path, list[LocalBlob], set[str], set[str]]:
    helper_roots = resolve_publication_branch_roots()
    if helper_roots.standalone_root != roots.standalone_root:
        raise fail(
            f"prepare_github_publication_branch roots mismatch: {helper_roots.standalone_root} != {roots.standalone_root}"
        )
    snapshot_root, snapshot_blobs, local_directories, local_top_level = (
        materialize_tracked_publication_branch(helper_roots)
    )
    blobs = [
        LocalBlob(
            path=blob.path,
            mode=blob.mode,
            blob_sha=git_blob_sha(blob.data),
            data=blob.data,
        )
        for blob in snapshot_blobs
    ]
    return snapshot_root, blobs, local_directories, local_top_level


def fetch_repository() -> dict[str, Any] | None:
    response = gh_api(f"repos/{REPOSITORY_FULL_NAME}", allow_404=True)
    if response is None:
        return None
    if not isinstance(response, dict):
        raise fail("repository payload must be an object")
    return response


def create_repository_if_missing() -> tuple[dict[str, Any], bool]:
    repository = fetch_repository()
    if repository is not None:
        return repository, False

    created = gh_api(
        "user/repos",
        method="POST",
        payload={"name": REPOSITORY_NAME, "private": False, "auto_init": False},
    )
    if not isinstance(created, dict):
        raise fail("repository creation returned invalid payload")
    return created, True


def ensure_repository_identity(repository: dict[str, Any]) -> None:
    if repository.get("full_name") != REPOSITORY_FULL_NAME:
        raise fail(f"unexpected repository full_name: {repository.get('full_name')}")
    if repository.get("visibility") != "public":
        raise fail(f"repository visibility must be public: {repository.get('visibility')}")
    if repository.get("default_branch") != BRANCH_NAME:
        raise fail(
            f"repository default branch must be {BRANCH_NAME}: {repository.get('default_branch')}"
        )


def fetch_ref() -> dict[str, Any] | None:
    endpoint = f"repos/{REPOSITORY_FULL_NAME}/git/ref/heads/{BRANCH_NAME}"
    completed = run(
        [
            "gh",
            "api",
            endpoint,
            "--method",
            "GET",
            "-H",
            "Accept: application/vnd.github+json",
        ],
        check=False,
    )
    if completed.returncode != 0:
        stderr = completed.stderr
        if "HTTP 404" in stderr or "Git Repository is empty." in stderr:
            return None
        raise CommandFailure(
            f"gh api failed: {endpoint}\nstdout:\n{completed.stdout}\nstderr:\n{stderr}"
        )
    payload = json.loads(completed.stdout)
    if not isinstance(payload, dict):
        raise fail("ref payload must be an object")
    return payload


def fetch_commit(commit_sha: str) -> dict[str, Any]:
    payload = gh_api(f"repos/{REPOSITORY_FULL_NAME}/git/commits/{commit_sha}")
    if not isinstance(payload, dict):
        raise fail(f"commit payload missing for {commit_sha}")
    return payload


def fetch_tree_recursive(tree_sha: str) -> dict[str, Any]:
    payload = gh_api(f"repos/{REPOSITORY_FULL_NAME}/git/trees/{tree_sha}?recursive=1")
    if not isinstance(payload, dict):
        raise fail(f"tree payload missing for {tree_sha}")
    if payload.get("truncated"):
        raise fail(f"truncated tree is forbidden: {tree_sha}")
    return payload


def collect_remote_snapshot(tree: dict[str, Any]) -> tuple[list[RemoteBlob], set[str], set[str]]:
    remote_files: list[RemoteBlob] = []
    remote_directories: set[str] = set()
    top_level: set[str] = set()

    tree_entries = tree.get("tree")
    if not isinstance(tree_entries, list):
        raise fail("tree payload missing tree list")

    for entry in tree_entries:
        if not isinstance(entry, dict):
            raise fail("tree entry must be an object")
        path = entry.get("path")
        entry_type = entry.get("type")
        mode = entry.get("mode")
        if not isinstance(path, str) or not path:
            raise fail(f"invalid tree path: {entry}")
        top_level.add(path.split("/", 1)[0])
        if entry_type == "tree":
            if mode != "040000":
                raise fail(f"unexpected tree mode for {path}: {mode}")
            remote_directories.add(path)
            continue
        if entry_type != "blob":
            raise fail(f"unexpected tree entry type for {path}: {entry_type}")
        if mode not in {"100644", "100755"}:
            raise fail(f"unexpected blob mode for {path}: {mode}")
        sha = entry.get("sha")
        if not isinstance(sha, str) or not sha:
            raise fail(f"missing blob sha for {path}")
        remote_files.append(RemoteBlob(path=path, mode=mode, blob_sha=sha))

    return sorted(remote_files, key=lambda item: item.path), remote_directories, top_level


def list_releases() -> list[dict[str, Any]]:
    payload = gh_api(f"repos/{REPOSITORY_FULL_NAME}/releases?per_page=100")
    if payload is None:
        return []
    if not isinstance(payload, list):
        raise fail("release list payload must be an array")
    for item in payload:
        if not isinstance(item, dict):
            raise fail("release item must be an object")
    return payload


def list_tags() -> list[dict[str, Any]]:
    payload = gh_api(f"repos/{REPOSITORY_FULL_NAME}/tags?per_page=100")
    if payload is None:
        return []
    if not isinstance(payload, list):
        raise fail("tag list payload must be an array")
    for item in payload:
        if not isinstance(item, dict):
            raise fail("tag item must be an object")
    return payload


def bootstrap_repository(bootstrap_readme_source: Path) -> str:
    bootstrap_bytes = bootstrap_readme_source.read_bytes()
    response = gh_api(
        f"repos/{REPOSITORY_FULL_NAME}/contents/README.md",
        method="PUT",
        payload={
            "message": BOOTSTRAP_MESSAGE,
            "content": base64.b64encode(bootstrap_bytes).decode("ascii"),
            "branch": BRANCH_NAME,
        },
    )
    if not isinstance(response, dict):
        raise fail("bootstrap response missing payload")
    current_ref = fetch_ref()
    if current_ref is None:
        raise fail("bootstrap did not create main ref")
    ref_object = current_ref.get("object")
    if not isinstance(ref_object, dict):
        raise fail("bootstrap ref missing object")
    sha = ref_object.get("sha")
    if not isinstance(sha, str) or not sha:
        raise fail("bootstrap ref missing sha")
    return sha


def assert_bootstrap_residue(
    tree: dict[str, Any],
    bootstrap_readme_source: Path,
    releases: list[dict[str, Any]],
    tags: list[dict[str, Any]],
) -> None:
    remote_files, remote_directories, remote_top_level = collect_remote_snapshot(tree)
    if remote_top_level != {"README.md"}:
        raise fail(f"bootstrap residue top-level mismatch: {sorted(remote_top_level)}")
    if remote_directories:
        raise fail(f"bootstrap residue must not contain directories: {sorted(remote_directories)}")
    if len(remote_files) != 1 or remote_files[0].path != "README.md":
        raise fail("bootstrap residue must contain only README.md")
    bootstrap_sha = git_blob_sha(bootstrap_readme_source.read_bytes())
    if remote_files[0].blob_sha != bootstrap_sha:
        raise fail("bootstrap residue README bytes do not match local publication-unit README")
    if releases:
        raise fail("bootstrap residue must have zero releases")
    if tags:
        raise fail("bootstrap residue must have zero tags")


def assert_allowed_release_and_tag_sets(
    releases: list[dict[str, Any]],
    tags: list[dict[str, Any]],
) -> None:
    allowed_tags = {RELEASE_TAG, *COMPATIBILITY_RELEASE_TAGS}
    seen_releases: set[str] = set()
    seen_tags: set[str] = set()

    for release in releases:
        release_tag = release.get("tag_name")
        if release_tag not in allowed_tags:
            raise fail(f"unexpected pre-existing release tag: {release.get('tag_name')}")
        if release_tag in seen_releases:
            raise fail(f"duplicate pre-existing release tag: {release_tag}")
        seen_releases.add(release_tag)

    for tag in tags:
        tag_name = tag.get("name")
        if tag_name not in allowed_tags:
            raise fail(f"unexpected pre-existing tag: {tag.get('name')}")
        if tag_name in seen_tags:
            raise fail(f"duplicate pre-existing tag: {tag_name}")
        seen_tags.add(tag_name)


def assert_allowed_preexisting_tracked_surface(
    tree: dict[str, Any],
    local_blobs: list[LocalBlob],
    local_directories: set[str],
    local_top_level: set[str],
) -> None:
    remote_files, remote_directories, remote_top_level = collect_remote_snapshot(tree)
    local_path_set = {blob.path for blob in local_blobs}
    remote_path_set = {blob.path for blob in remote_files}

    if remote_top_level != local_top_level:
        raise fail(
            f"pre-existing tracked top-level mismatch: {sorted(remote_top_level)} != {sorted(local_top_level)}"
        )
    if remote_directories != local_directories:
        raise fail(
            f"pre-existing tracked directory set mismatch: {sorted(remote_directories)} != {sorted(local_directories)}"
        )
    if remote_path_set != local_path_set:
        raise fail(
            f"pre-existing tracked path set mismatch: {sorted(remote_path_set)} != {sorted(local_path_set)}"
        )


def assert_allowed_preexisting_state(
    tree: dict[str, Any],
    bootstrap_readme_source: Path,
    local_blobs: list[LocalBlob],
    local_directories: set[str],
    local_top_level: set[str],
    releases: list[dict[str, Any]],
    tags: list[dict[str, Any]],
) -> str:
    try:
        assert_bootstrap_residue(tree, bootstrap_readme_source, releases, tags)
        return "bootstrap_residue"
    except RuntimeError:
        pass

    assert_allowed_release_and_tag_sets(releases, tags)
    assert_allowed_preexisting_tracked_surface(
        tree,
        local_blobs,
        local_directories,
        local_top_level,
    )
    return "tracked_surface_normalizable"


def create_blob(data: bytes, expected_sha: str) -> str:
    response = gh_api(
        f"repos/{REPOSITORY_FULL_NAME}/git/blobs",
        method="POST",
        payload={
            "content": base64.b64encode(data).decode("ascii"),
            "encoding": "base64",
        },
    )
    if not isinstance(response, dict):
        raise fail("blob creation returned invalid payload")
    sha = response.get("sha")
    if sha != expected_sha:
        raise fail(f"blob sha mismatch: {sha} != {expected_sha}")
    return expected_sha


def create_tree(local_blobs: list[LocalBlob]) -> str:
    tree_entries = []
    for blob in local_blobs:
        if blob.mode not in {"100644", "100755"}:
            raise fail(f"unexpected local blob mode for {blob.path}: {blob.mode}")
        tree_entries.append(
            {
                "path": blob.path,
                "mode": blob.mode,
                "type": "blob",
                "sha": create_blob(blob.data, blob.blob_sha),
            }
        )
    response = gh_api(
        f"repos/{REPOSITORY_FULL_NAME}/git/trees",
        method="POST",
        payload={"tree": tree_entries},
    )
    if not isinstance(response, dict) or not isinstance(response.get("sha"), str):
        raise fail("tree creation returned invalid payload")
    return response["sha"]


def create_commit(tree_sha: str, parent_sha: str) -> str:
    response = gh_api(
        f"repos/{REPOSITORY_FULL_NAME}/git/commits",
        method="POST",
        payload={
            "message": COMMIT_MESSAGE,
            "tree": tree_sha,
            "parents": [parent_sha],
        },
    )
    if not isinstance(response, dict) or not isinstance(response.get("sha"), str):
        raise fail("commit creation returned invalid payload")
    return response["sha"]


def update_main_ref(commit_sha: str) -> None:
    gh_api(
        f"repos/{REPOSITORY_FULL_NAME}/git/refs/heads/{BRANCH_NAME}",
        method="PATCH",
        payload={"sha": commit_sha, "force": False},
    )


def assert_exact_tracked_surface(
    local_blobs: list[LocalBlob],
    local_directories: set[str],
    local_top_level: set[str],
    tree: dict[str, Any],
) -> None:
    remote_files, remote_directories, remote_top_level = collect_remote_snapshot(tree)
    local_file_map = {blob.path: (blob.mode, blob.blob_sha) for blob in local_blobs}
    remote_file_map = {blob.path: (blob.mode, blob.blob_sha) for blob in remote_files}

    if remote_top_level != local_top_level:
        raise fail(
            f"remote top-level set mismatch: {sorted(remote_top_level)} != {sorted(local_top_level)}"
        )
    if remote_directories != local_directories:
        raise fail(
            f"remote directory set mismatch: {sorted(remote_directories)} != {sorted(local_directories)}"
        )
    if remote_file_map != local_file_map:
        missing = sorted(set(local_file_map) - set(remote_file_map))
        extra = sorted(set(remote_file_map) - set(local_file_map))
        changed = sorted(
            path
            for path in set(local_file_map) & set(remote_file_map)
            if local_file_map[path] != remote_file_map[path]
        )
        raise fail(
            "remote tracked surface mismatch: "
            f"missing={missing}, extra={extra}, changed={changed}"
        )


def is_exact_tracked_surface(
    local_blobs: list[LocalBlob],
    local_directories: set[str],
    local_top_level: set[str],
    tree: dict[str, Any],
) -> bool:
    try:
        assert_exact_tracked_surface(local_blobs, local_directories, local_top_level, tree)
        return True
    except RuntimeError:
        return False


def ensure_tag_ref(commit_sha: str) -> None:
    tag_ref = gh_api(
        f"repos/{REPOSITORY_FULL_NAME}/git/ref/tags/{RELEASE_TAG}",
        allow_404=True,
    )
    if tag_ref is None:
        gh_api(
            f"repos/{REPOSITORY_FULL_NAME}/git/refs",
            method="POST",
            payload={"ref": f"refs/tags/{RELEASE_TAG}", "sha": commit_sha},
        )
        return
    if not isinstance(tag_ref, dict):
        raise fail("tag ref payload must be an object")
    object_payload = tag_ref.get("object")
    if not isinstance(object_payload, dict):
        raise fail("tag ref object payload missing")
    current_sha = object_payload.get("sha")
    if current_sha == commit_sha:
        return
    gh_api(
        f"repos/{REPOSITORY_FULL_NAME}/git/refs/tags/{RELEASE_TAG}",
        method="PATCH",
        payload={"sha": commit_sha, "force": True},
    )


def delete_release_if_exists() -> None:
    release = fetch_release_by_tag()
    if release is None:
        return
    release_id = release.get("id")
    if not isinstance(release_id, int):
        raise fail("release id missing during rollback")
    gh_api(
        f"repos/{REPOSITORY_FULL_NAME}/releases/{release_id}",
        method="DELETE",
    )


def delete_tag_if_exists() -> None:
    tag_ref = gh_api(
        f"repos/{REPOSITORY_FULL_NAME}/git/ref/tags/{RELEASE_TAG}",
        allow_404=True,
    )
    if tag_ref is None:
        return
    gh_api(
        f"repos/{REPOSITORY_FULL_NAME}/git/refs/tags/{RELEASE_TAG}",
        method="DELETE",
    )


def restore_main_ref(commit_sha: str) -> None:
    gh_api(
        f"repos/{REPOSITORY_FULL_NAME}/git/refs/heads/{BRANCH_NAME}",
        method="PATCH",
        payload={"sha": commit_sha, "force": True},
    )


def rollback_to_bootstrap_state(commit_sha: str) -> None:
    last_error: Exception | None = None
    for attempt in range(ROLLBACK_RETRY_COUNT):
        try:
            delete_release_if_exists()
            delete_tag_if_exists()
            restore_main_ref(commit_sha)

            current_ref = fetch_ref()
            if current_ref is None:
                raise fail("main ref missing during rollback verification")
            current_sha = current_ref.get("object", {}).get("sha")
            if current_sha != commit_sha:
                raise fail(f"main ref rollback mismatch: {current_sha} != {commit_sha}")

            release = fetch_release_by_tag()
            if release is not None:
                raise fail("release residue remains after rollback")

            tag_ref = gh_api(
                f"repos/{REPOSITORY_FULL_NAME}/git/ref/tags/{RELEASE_TAG}",
                allow_404=True,
            )
            if tag_ref is not None:
                raise fail("tag residue remains after rollback")
            return
        except Exception as exc:  # pragma: no cover - defensive rollback path
            last_error = exc
            if attempt + 1 == ROLLBACK_RETRY_COUNT:
                break
            time.sleep(ROLLBACK_RETRY_DELAY_SECONDS)
    raise fail(f"rollback to bootstrap state failed: {last_error}")


def download_url_to_path(url: str, destination: Path) -> None:
    last_error: Exception | None = None
    ssl_context = build_ssl_context()
    destination.parent.mkdir(parents=True, exist_ok=True)
    for attempt in range(URL_RETRY_COUNT):
        request = Request(url, method="GET")
        try:
            with urlopen(request, context=ssl_context, timeout=60) as response:
                if response.status != 200:
                    raise fail(f"url not reachable: {url} status={response.status}")
                with destination.open("wb") as fh:
                    while True:
                        chunk = response.read(1024 * 1024)
                        if not chunk:
                            break
                        fh.write(chunk)
            return
        except (HTTPError, URLError, TimeoutError, RuntimeError) as exc:
            last_error = exc
            if destination.exists():
                destination.unlink()
            if attempt + 1 == URL_RETRY_COUNT:
                break
            time.sleep(URL_RETRY_DELAY_SECONDS)
    raise fail(f"url download failed: {url} error={last_error}")


def capture_release_baseline(main_commit_sha: str, rollback_dir: Path) -> ReleaseBaseline:
    tag_ref = gh_api(
        f"repos/{REPOSITORY_FULL_NAME}/git/ref/tags/{RELEASE_TAG}",
        allow_404=True,
    )
    tag_sha: str | None = None
    if tag_ref is not None:
        if not isinstance(tag_ref, dict):
            raise fail("tag ref payload must be an object")
        object_payload = tag_ref.get("object")
        if not isinstance(object_payload, dict):
            raise fail("tag ref object payload missing")
        tag_sha = object_payload.get("sha")
        if not isinstance(tag_sha, str) or not tag_sha:
            raise fail("tag ref sha missing")

    release = fetch_release_by_tag()
    if release is None:
        return ReleaseBaseline(
            main_commit_sha=main_commit_sha,
            tag_sha=tag_sha,
            release_exists=False,
            rollback_asset_path=None,
        )
    if tag_sha is None:
        raise fail("pre-mutation release exists without tag baseline")

    assets = collect_release_assets(release)
    if len(assets) != 1:
        raise fail(f"pre-mutation release asset set must contain exactly one asset: {len(assets)}")
    asset = assets[0]
    if asset.name != ASSET_FILENAME:
        raise fail(f"unexpected pre-mutation asset filename: {asset.name}")

    rollback_asset_path = rollback_dir / asset.name
    download_url_to_path(asset.download_url, rollback_asset_path)
    _, downloaded_size = hash_file(rollback_asset_path)
    if downloaded_size != asset.size:
        raise fail(f"pre-mutation asset size mismatch: {downloaded_size} != {asset.size}")

    return ReleaseBaseline(
        main_commit_sha=main_commit_sha,
        tag_sha=tag_sha,
        release_exists=True,
        rollback_asset_path=rollback_asset_path,
    )


def rollback_release_surface(baseline: ReleaseBaseline) -> None:
    last_error: Exception | None = None
    for attempt in range(ROLLBACK_RETRY_COUNT):
        try:
            delete_release_if_exists()
            delete_tag_if_exists()

            if baseline.tag_sha is not None:
                ensure_tag_ref(baseline.tag_sha)

            if baseline.release_exists:
                if baseline.tag_sha is None:
                    raise fail("release baseline requires tag baseline")
                if baseline.rollback_asset_path is None:
                    raise fail("release rollback asset missing")
                restored_release = ensure_release(baseline.tag_sha)
                delete_stale_assets(restored_release)
                upload_asset(baseline.rollback_asset_path)
            restored_release = fetch_release_by_tag()
            restored_tag_ref = gh_api(
                f"repos/{REPOSITORY_FULL_NAME}/git/ref/tags/{RELEASE_TAG}",
                allow_404=True,
            )

            current_ref = fetch_ref()
            if current_ref is None:
                raise fail("main ref missing during release rollback verification")
            current_main_sha = current_ref.get("object", {}).get("sha")
            if current_main_sha != baseline.main_commit_sha:
                raise fail(
                    f"branch mutation detected during release rollback: {current_main_sha} != {baseline.main_commit_sha}"
                )

            if baseline.tag_sha is None:
                if restored_tag_ref is not None:
                    raise fail("tag residue remains after release rollback")
            else:
                if not isinstance(restored_tag_ref, dict):
                    raise fail("restored tag ref payload must be an object")
                restored_tag_sha = restored_tag_ref.get("object", {}).get("sha")
                if restored_tag_sha != baseline.tag_sha:
                    raise fail(
                        f"release rollback tag mismatch: {restored_tag_sha} != {baseline.tag_sha}"
                    )

            if baseline.release_exists:
                if restored_release is None:
                    raise fail("release missing after release rollback")
                assert_release_state(restored_release, baseline.rollback_asset_path)
            elif restored_release is not None:
                raise fail("release residue remains after release rollback")
            return
        except Exception as exc:  # pragma: no cover - defensive rollback path
            last_error = exc
            if attempt + 1 == ROLLBACK_RETRY_COUNT:
                break
            time.sleep(ROLLBACK_RETRY_DELAY_SECONDS)
    raise fail(f"release rollback failed: {last_error}")


def fetch_release_by_tag() -> dict[str, Any] | None:
    payload = gh_api(
        f"repos/{REPOSITORY_FULL_NAME}/releases/tags/{RELEASE_TAG}",
        allow_404=True,
    )
    if payload is None:
        return None
    if not isinstance(payload, dict):
        raise fail("release payload must be an object")
    return payload


def ensure_release(commit_sha: str) -> dict[str, Any]:
    release = fetch_release_by_tag()
    payload = {
        "tag_name": RELEASE_TAG,
        "target_commitish": commit_sha,
        "name": RELEASE_TITLE,
        "body": RELEASE_BODY,
        "draft": False,
        "prerelease": False,
    }
    if release is None:
        created = gh_api(
            f"repos/{REPOSITORY_FULL_NAME}/releases",
            method="POST",
            payload=payload,
        )
        if not isinstance(created, dict):
            raise fail("release creation returned invalid payload")
        return created

    release_id = release.get("id")
    if not isinstance(release_id, int):
        raise fail("release id missing")
    updated = gh_api(
        f"repos/{REPOSITORY_FULL_NAME}/releases/{release_id}",
        method="PATCH",
        payload=payload,
    )
    if not isinstance(updated, dict):
        raise fail("release update returned invalid payload")
    return updated


def collect_release_assets(release: dict[str, Any]) -> list[ReleaseAsset]:
    raw_assets = release.get("assets")
    if not isinstance(raw_assets, list):
        raise fail("release assets payload missing")
    assets: list[ReleaseAsset] = []
    for item in raw_assets:
        if not isinstance(item, dict):
            raise fail("release asset must be an object")
        asset_id = item.get("id")
        name = item.get("name")
        size = item.get("size")
        download_url = item.get("browser_download_url")
        if not isinstance(asset_id, int):
            raise fail("release asset id missing")
        if not isinstance(name, str):
            raise fail("release asset name missing")
        if not isinstance(size, int):
            raise fail("release asset size missing")
        if not isinstance(download_url, str):
            raise fail("release asset url missing")
        assets.append(
            ReleaseAsset(
                asset_id=asset_id,
                name=name,
                size=size,
                download_url=download_url,
            )
        )
    return assets


def delete_stale_assets(release: dict[str, Any]) -> None:
    for asset in collect_release_assets(release):
        if asset.name == ASSET_FILENAME:
            continue
        gh_api(
            f"repos/{REPOSITORY_FULL_NAME}/releases/assets/{asset.asset_id}",
            method="DELETE",
        )


def upload_asset(package_asset: Path) -> None:
    run(
        [
            "gh",
            "release",
            "upload",
            RELEASE_TAG,
            str(package_asset),
            "--clobber",
            "--repo",
            REPOSITORY_FULL_NAME,
        ]
    )


def hash_file(path: Path) -> tuple[str, int]:
    hasher = hashlib.sha256()
    size = 0
    with path.open("rb") as fh:
        while True:
            chunk = fh.read(1024 * 1024)
            if not chunk:
                break
            hasher.update(chunk)
            size += len(chunk)
    return hasher.hexdigest(), size


def load_release_manifest(package_asset: Path) -> dict[str, Any]:
    with tarfile.open(package_asset, "r:gz") as archive:
        try:
            member = archive.getmember("cyrune-free-v0.1/RELEASE_MANIFEST.json")
        except KeyError as exc:
            raise fail("missing RELEASE_MANIFEST.json in carrier asset") from exc
        extracted = archive.extractfile(member)
        if extracted is None:
            raise fail("failed to extract RELEASE_MANIFEST.json from carrier asset")
        payload = json.loads(extracted.read().decode("utf-8"))
    if not isinstance(payload, dict):
        raise fail("RELEASE_MANIFEST.json must be an object")
    return payload


def compute_sha256(path: Path) -> str:
    hasher = hashlib.sha256()
    with path.open("rb") as fh:
        while True:
            chunk = fh.read(1024 * 1024)
            if not chunk:
                break
            hasher.update(chunk)
    return hasher.hexdigest()


def parse_sha256sums(content: str) -> dict[str, str]:
    entries: dict[str, str] = {}
    for raw_line in content.splitlines():
        line = raw_line.strip()
        if not line:
            continue
        digest, separator, relative_path = raw_line.partition("  ")
        if separator != "  " or not digest or not relative_path:
            raise fail(f"invalid SHA256SUMS line: {raw_line!r}")
        if relative_path in entries:
            raise fail(f"duplicate SHA256SUMS path: {relative_path}")
        entries[relative_path] = digest
    if not entries:
        raise fail("empty SHA256SUMS is forbidden")
    return entries


def parse_single_hash_line(content: str, *, expected_name: str) -> str:
    lines = [line.strip() for line in content.splitlines() if line.strip()]
    if len(lines) != 1:
        raise fail("archive hash sidecar must contain exactly one line")
    digest, separator, name = lines[0].partition("  ")
    if separator != "  " or not digest or name != expected_name:
        raise fail(f"invalid archive hash sidecar line: {lines[0]!r}")
    return digest


def collect_directory_file_hashes(root: Path) -> dict[str, str]:
    if not root.exists() or not root.is_dir() or root.is_symlink():
        raise fail(f"invalid package extract root: {root}")
    hashes: dict[str, str] = {}
    for path in sorted(root.rglob("*")):
        if path.is_symlink():
            raise fail(f"symlink extract-root entry is forbidden: {path}")
        if path.is_dir():
            continue
        if not path.is_file():
            raise fail(f"non-regular extract-root entry is forbidden: {path}")
        relative_path = path.relative_to(root).as_posix()
        if relative_path in hashes:
            raise fail(f"duplicate extract-root entry: {relative_path}")
        hashes[relative_path] = compute_sha256(path)
    return hashes


def collect_archive_file_hashes(package_asset: Path) -> dict[str, str]:
    hashes: dict[str, str] = {}
    with tarfile.open(package_asset, "r:gz") as archive:
        for member in archive.getmembers():
            if member.issym() or member.islnk():
                raise fail(f"symlink archive member is forbidden: {member.name}")
            if member.isdir():
                continue
            if not member.isfile():
                raise fail(f"non-regular archive member is forbidden: {member.name}")
            if member.name in hashes:
                raise fail(f"duplicate archive member is forbidden: {member.name}")
            extracted = archive.extractfile(member)
            if extracted is None:
                raise fail(f"failed to extract archive member: {member.name}")
            hasher = hashlib.sha256()
            while True:
                chunk = extracted.read(1024 * 1024)
                if not chunk:
                    break
                hasher.update(chunk)
            hashes[member.name] = hasher.hexdigest()
    return hashes


def assert_local_carrier_contract(package_asset: Path) -> dict[str, Any]:
    release_manifest = load_release_manifest(package_asset)
    expected_pairs = {
        "distribution_unit": ASSET_FILENAME,
        "integrity_mode": "sha256",
        "signature_mode": "macos-adhoc",
        "update_policy": "fixed-distribution/no-self-update",
    }
    for key, expected_value in expected_pairs.items():
        if release_manifest.get(key) != expected_value:
            raise fail(
                f"carrier manifest mismatch for {key}: {release_manifest.get(key)!r} != {expected_value!r}"
            )

    package_root = package_asset.parent / "cyrune-free-v0.1"
    checksum_manifest_path = package_root / "SHA256SUMS.txt"
    checksum_entries = parse_sha256sums(checksum_manifest_path.read_text(encoding="utf-8"))
    archive_hash_sidecar = package_asset.parent / "guard" / "archive-sha256.txt"
    archive_self_hash = parse_single_hash_line(
        archive_hash_sidecar.read_text(encoding="utf-8"),
        expected_name=package_asset.name,
    )

    extract_root_hashes = collect_directory_file_hashes(package_root)
    if "SHA256SUMS.txt" not in extract_root_hashes:
        raise fail("extract-root missing SHA256SUMS.txt")
    expected_extract_paths = set(checksum_entries) | {"SHA256SUMS.txt"}
    if set(extract_root_hashes) != expected_extract_paths:
        missing_extract_files = sorted(expected_extract_paths - set(extract_root_hashes))
        extra_extract_files = sorted(set(extract_root_hashes) - expected_extract_paths)
        raise fail(
            "extract-root exact set mismatch: "
            f"missing={missing_extract_files} extra={extra_extract_files}"
        )
    for relative_path, expected_hash in checksum_entries.items():
        actual_hash = extract_root_hashes[relative_path]
        if actual_hash != expected_hash:
            raise fail(
                f"extract-root hash mismatch for {relative_path}: {actual_hash} != {expected_hash}"
            )

    actual_archive_hash, asset_size = hash_file(package_asset)
    if actual_archive_hash != archive_self_hash:
        raise fail(
            f"archive self-hash mismatch: {actual_archive_hash} != {archive_self_hash}"
        )

    required_members = {
        "cyrune-free-v0.1/share/cyrune/bundle-root/embedding/artifacts/multilingual-e5-small/model.onnx",
        "cyrune-free-v0.1/share/cyrune/home-template/embedding/artifacts/multilingual-e5-small/model.onnx",
    }
    archive_hashes = collect_archive_file_hashes(package_asset)
    expected_archive_hashes = {
        f"cyrune-free-v0.1/{relative_path}": digest
        for relative_path, digest in checksum_entries.items()
    }
    expected_archive_hashes["cyrune-free-v0.1/SHA256SUMS.txt"] = extract_root_hashes[
        "SHA256SUMS.txt"
    ]
    if set(archive_hashes) != set(expected_archive_hashes):
        missing_archive_members = sorted(set(expected_archive_hashes) - set(archive_hashes))
        extra_archive_members = sorted(set(archive_hashes) - set(expected_archive_hashes))
        raise fail(
            "archive exact member set mismatch: "
            f"missing={missing_archive_members} extra={extra_archive_members}"
        )
    for member_name, expected_hash in expected_archive_hashes.items():
        actual_hash = archive_hashes[member_name]
        if actual_hash != expected_hash:
            raise fail(
                f"archive member hash mismatch for {member_name}: {actual_hash} != {expected_hash}"
            )

    member_names = set(archive_hashes)
    missing_members = sorted(required_members - member_names)
    if missing_members:
        raise fail(f"carrier-only payload missing from asset: {missing_members}")

    return {
        "package_asset": str(package_asset),
        "distribution_unit": release_manifest["distribution_unit"],
        "archive_sha256": actual_archive_hash,
        "asset_size": asset_size,
        "extract_root_path_count": len(extract_root_hashes),
        "archive_member_count": len(archive_hashes),
        "required_members": sorted(required_members),
        "archive_hash_sidecar": str(archive_hash_sidecar),
    }


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Publish or validate the CYRUNE GitHub release package surface."
    )
    mode_group = parser.add_mutually_exclusive_group(required=True)
    mode_group.add_argument(
        "--check-local-carrier",
        action="store_true",
        help="Validate the local carrier contract without any remote mutation.",
    )
    mode_group.add_argument(
        "--publish-branch",
        action="store_true",
        help="Reserved for OP-5 remote tracked branch publication.",
    )
    mode_group.add_argument(
        "--publish-release",
        action="store_true",
        help="Reserved for OP-6 remote release carrier publication.",
    )
    return parser.parse_args()


def verify_page_url(url: str) -> None:
    last_error: Exception | None = None
    ssl_context = build_ssl_context()
    for attempt in range(URL_RETRY_COUNT):
        request = Request(url, method="GET")
        try:
            with urlopen(request, context=ssl_context, timeout=30) as response:
                if response.status != 200:
                    raise fail(f"url not reachable: {url} status={response.status}")
                response.read(1)
                return
        except (HTTPError, URLError, TimeoutError) as exc:
            last_error = exc
            if attempt + 1 == URL_RETRY_COUNT:
                break
            time.sleep(URL_RETRY_DELAY_SECONDS)
    raise fail(f"url reachability check failed: {url} error={last_error}")


def verify_asset_url(local_package: Path) -> tuple[str, int]:
    expected_hash, expected_size = hash_file(local_package)
    last_error: Exception | None = None
    ssl_context = build_ssl_context()
    for attempt in range(URL_RETRY_COUNT):
        try:
            request = Request(EXACT_ASSET_URL, method="GET")
            hasher = hashlib.sha256()
            size = 0
            with urlopen(request, context=ssl_context, timeout=60) as response:
                if response.status != 200:
                    raise fail(
                        f"exact asset url not reachable: {EXACT_ASSET_URL} status={response.status}"
                    )
                while True:
                    chunk = response.read(1024 * 1024)
                    if not chunk:
                        break
                    hasher.update(chunk)
                    size += len(chunk)
            if size != expected_size:
                raise fail(f"asset size mismatch: {size} != {expected_size}")
            actual_hash = hasher.hexdigest()
            if actual_hash != expected_hash:
                raise fail(f"asset hash mismatch: {actual_hash} != {expected_hash}")
            return expected_hash, expected_size
        except (HTTPError, URLError, TimeoutError, RuntimeError) as exc:
            last_error = exc
            if attempt + 1 == URL_RETRY_COUNT:
                break
            time.sleep(URL_RETRY_DELAY_SECONDS)
    raise fail(f"exact asset verification failed: {last_error}")


def assert_release_state(release: dict[str, Any], package_asset: Path) -> dict[str, Any]:
    if release.get("tag_name") != RELEASE_TAG:
        raise fail(f"unexpected release tag: {release.get('tag_name')}")
    if release.get("name") != RELEASE_TITLE:
        raise fail(f"unexpected release title: {release.get('name')}")
    if release.get("body") != RELEASE_BODY:
        raise fail(f"unexpected release body: {release.get('body')!r}")
    if release.get("html_url") != RELEASE_LANDING_PAGE:
        raise fail(f"unexpected release landing page: {release.get('html_url')}")
    if release.get("draft") is not False:
        raise fail("draft release is forbidden")
    if release.get("prerelease") is not False:
        raise fail("prerelease is forbidden")

    assets = collect_release_assets(release)
    if len(assets) != 1:
        raise fail(f"release asset set must contain exactly one asset: {len(assets)}")
    asset = assets[0]
    if asset.name != ASSET_FILENAME:
        raise fail(f"unexpected asset filename: {asset.name}")
    if asset.download_url != EXACT_ASSET_URL:
        raise fail(f"unexpected asset url: {asset.download_url}")
    _, expected_size = hash_file(package_asset)
    if asset.size != expected_size:
        raise fail(f"unexpected asset size: {asset.size} != {expected_size}")

    return {
        "asset_name": asset.name,
        "asset_size": asset.size,
        "asset_url": asset.download_url,
    }


def publish_release_surface() -> None:
    roots = resolve_roots(
        require_publication_roots=False,
        require_package_asset=True,
    )
    ensure_host_prerequisite()

    current_ref = fetch_ref()
    if current_ref is None:
        raise fail("main ref missing before release publication")
    main_commit_sha = current_ref.get("object", {}).get("sha")
    if not isinstance(main_commit_sha, str) or not main_commit_sha:
        raise fail("main ref sha missing before release publication")

    with tempfile.TemporaryDirectory(prefix="cyrune-release-rollback-") as rollback_dir_raw:
        release_baseline = capture_release_baseline(main_commit_sha, Path(rollback_dir_raw))
        mutation_started = False
        try:
            mutation_started = True
            ensure_tag_ref(main_commit_sha)
            release = ensure_release(main_commit_sha)
            delete_stale_assets(release)
            upload_asset(roots.package_asset)

            release = fetch_release_by_tag()
            if release is None:
                raise fail(f"missing release after upload: {RELEASE_TAG}")
            release_summary = assert_release_state(release, roots.package_asset)

            current_ref = fetch_ref()
            if current_ref is None:
                raise fail("main ref missing after release publication")
            current_main_sha = current_ref.get("object", {}).get("sha")
            if current_main_sha != main_commit_sha:
                raise fail(
                    f"branch mutation detected during release publication: {current_main_sha} != {main_commit_sha}"
                )

            verify_page_url(REPOSITORY_ROOT_PAGE)
            verify_page_url(RELEASE_LANDING_PAGE)
            asset_hash, asset_size = verify_asset_url(roots.package_asset)
        except Exception:
            if mutation_started:
                rollback_release_surface(release_baseline)
            raise

    print(
        json.dumps(
            {
                "repository_full_name": REPOSITORY_FULL_NAME,
                "branch_ref": BRANCH_REF,
                "branch_commit_sha": main_commit_sha,
                "release_tag": RELEASE_TAG,
                "release_landing_page": RELEASE_LANDING_PAGE,
                "exact_asset_url": EXACT_ASSET_URL,
                "asset_sha256": asset_hash,
                "asset_size": asset_size,
                "release_summary": release_summary,
            },
            ensure_ascii=True,
        )
    )


def publish_branch_surface() -> None:
    roots = resolve_roots(require_package_asset=False)
    ensure_host_prerequisite()
    snapshot_root, local_blobs, local_directories, local_top_level = load_local_tracked_blobs(
        roots
    )

    repository, created = create_repository_if_missing()
    current_ref = fetch_ref()
    if created:
        bootstrap_repository(snapshot_root / "README.md")
        repository = fetch_repository()
        if repository is None:
            raise fail(f"missing repository after creation: {REPOSITORY_FULL_NAME}")
        current_ref = fetch_ref()

    ensure_repository_identity(repository)
    if current_ref is None:
        raise fail("pre-existing empty repository is forbidden")

    commit_sha = current_ref.get("object", {}).get("sha")
    if not isinstance(commit_sha, str) or not commit_sha:
        raise fail("current ref sha missing")

    releases = list_releases()
    tags = list_tags()
    current_commit = fetch_commit(commit_sha)
    tree_sha = current_commit.get("tree", {}).get("sha")
    if not isinstance(tree_sha, str) or not tree_sha:
        raise fail("current commit tree sha missing")
    current_tree = fetch_tree_recursive(tree_sha)
    preexisting_mode = assert_allowed_preexisting_state(
        current_tree,
        snapshot_root / "README.md",
        local_blobs,
        local_directories,
        local_top_level,
        releases,
        tags,
    )

    new_commit_sha: str | None = None
    mutated_main_ref = False
    try:
        if preexisting_mode == "tracked_surface_normalizable" and is_exact_tracked_surface(
            local_blobs,
            local_directories,
            local_top_level,
            current_tree,
        ):
            new_commit_sha = commit_sha
            updated_tree = current_tree
        else:
            new_tree_sha = create_tree(local_blobs)
            new_commit_sha = create_commit(new_tree_sha, commit_sha)
            update_main_ref(new_commit_sha)
            mutated_main_ref = True

            refreshed_ref = fetch_ref()
            if refreshed_ref is None:
                raise fail(f"missing ref after update: {BRANCH_REF}")
            refreshed_sha = refreshed_ref.get("object", {}).get("sha")
            if refreshed_sha != new_commit_sha:
                raise fail(f"unexpected ref sha after update: {refreshed_sha} != {new_commit_sha}")

            refreshed_repository = fetch_repository()
            if refreshed_repository is None:
                raise fail(f"missing repository after update: {REPOSITORY_FULL_NAME}")
            ensure_repository_identity(refreshed_repository)

            updated_commit = fetch_commit(new_commit_sha)
            updated_tree_sha = updated_commit.get("tree", {}).get("sha")
            if not isinstance(updated_tree_sha, str) or not updated_tree_sha:
                raise fail("updated commit tree sha missing")
            updated_tree = fetch_tree_recursive(updated_tree_sha)
        assert_exact_tracked_surface(local_blobs, local_directories, local_top_level, updated_tree)
    except Exception:
        if mutated_main_ref:
            restore_main_ref(commit_sha)
            rolled_back_ref = fetch_ref()
            if rolled_back_ref is None:
                raise fail("main ref missing during branch rollback verification")
            rolled_back_sha = rolled_back_ref.get("object", {}).get("sha")
            if rolled_back_sha != commit_sha:
                raise fail(
                    f"branch rollback mismatch: {rolled_back_sha} != {commit_sha}"
                )
        raise

    print(
        json.dumps(
            {
                "repository_full_name": REPOSITORY_FULL_NAME,
                "branch_ref": BRANCH_REF,
                "commit_sha": new_commit_sha,
                "clone_url": CLONE_URL,
                "preexisting_mode": preexisting_mode,
                "tracked_top_level_set": sorted(local_top_level),
                "tracked_path_set": sorted(blob.path for blob in local_blobs),
            },
            ensure_ascii=True,
        )
    )


def main() -> None:
    args = parse_args()
    if args.check_local_carrier:
        roots = resolve_roots(
            require_publication_roots=False,
            require_package_asset=True,
        )
        contract_summary = assert_local_carrier_contract(roots.package_asset)
        print(
            json.dumps(
                {"mode": "check-local-carrier", **contract_summary},
                ensure_ascii=True,
            )
        )
        return
    if args.publish_branch:
        publish_branch_surface()
        return
    if args.publish_release:
        publish_release_surface()
        return
    raise fail("exactly one subcommand is required")


if __name__ == "__main__":
    main()
