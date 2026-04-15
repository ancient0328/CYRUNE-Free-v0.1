#!/usr/bin/env python3
from __future__ import annotations

import json
import shutil
import stat
import sys
from dataclasses import dataclass
from pathlib import Path


TRACKED_TOP_LEVEL_ENTRY_SET = {"README.md", "docs", "scripts", "free"}
FORBIDDEN_PARTS = {"target", "__pycache__"}
FORBIDDEN_NAMES = {".DS_Store"}
FORBIDDEN_SUFFIXES = {".pyc"}
CARRIER_ONLY_EXCLUDED_RELATIVE_PATHS = {
    Path("free/v0.1/0/resources/bundle-root/embedding/artifacts/multilingual-e5-small/model.onnx")
}
REQUIRED_RELATIVE_PATHS = {
    Path("README.md"),
    Path("docs/CYRUNE_Free_Public_Index.md"),
    Path("docs/GETTING_STARTED.md"),
    Path("docs/TROUBLESHOOTING.md"),
    Path("scripts/prepare-public-run.sh"),
    Path("scripts/doctor.sh"),
    Path("scripts/first-success.sh"),
    Path("free/v0.1/0/Cargo.toml"),
    Path("free/v0.1/0/Cargo.lock"),
    Path("free/v0.1/dev-docs/summary/00-SUMMARY_INDEX.md"),
    Path("free/v0.1/dev-docs/00-TARGET_SYSTEM.md"),
    Path("free/v0.1/dev-docs/03-architecture/ARCHITECTURE_OVERVIEW.md"),
    Path(
        "free/v0.1/dev-docs/90-reports/20260410-terminal-D6-native-outer-launcher-proof.md"
    ),
    Path(
        "free/v0.1/dev-docs/90-reports/20260411-terminal-D7-terminal-bundle-productization-proof.md"
    ),
    Path(
        "free/v0.1/dev-docs/90-reports/20260412-terminal-EVID-D7RC1D-1-external-release-concretization-closeout.md"
    ),
}
TRACKED_SCRIPT_RELATIVE_PATHS = {
    Path("scripts/prepare-public-run.sh"),
    Path("scripts/doctor.sh"),
    Path("scripts/first-success.sh"),
}
TRACKED_DEV_DOC_EXACT_RELATIVE_PATHS = {
    Path("free/v0.1/dev-docs/00-TARGET_SYSTEM.md"),
    Path("free/v0.1/dev-docs/03-architecture/ARCHITECTURE_OVERVIEW.md"),
    Path(
        "free/v0.1/dev-docs/90-reports/20260410-terminal-D6-native-outer-launcher-proof.md"
    ),
    Path(
        "free/v0.1/dev-docs/90-reports/20260411-terminal-D7-terminal-bundle-productization-proof.md"
    ),
    Path(
        "free/v0.1/dev-docs/90-reports/20260412-terminal-EVID-D7RC1D-1-external-release-concretization-closeout.md"
    ),
}


def fail(message: str) -> RuntimeError:
    return RuntimeError(message)


@dataclass(frozen=True)
class Roots:
    script_path: Path
    script_dir: Path
    standalone_root: Path
    version_root: Path
    free_family_root: Path
    source_root: Path
    public_root: Path
    output_root: Path


@dataclass(frozen=True)
class SnapshotBlob:
    path: str
    mode: str
    data: bytes


def resolve_roots() -> Roots:
    script_path = Path(__file__).resolve()
    script_dir = script_path.parent
    standalone_root = script_dir.parent
    version_root = standalone_root.parent
    free_family_root = version_root.parent
    source_root = free_family_root.parent
    public_root = source_root / "public" / "free-v0.1"
    output_root = standalone_root / "target" / "shipping" / "S2" / "github-publication-branch"

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
    if not public_root.exists() or not public_root.is_dir() or public_root.is_symlink():
        raise fail(f"invalid PUBLIC_ROOT: {public_root}")

    return Roots(
        script_path=script_path,
        script_dir=script_dir,
        standalone_root=standalone_root,
        version_root=version_root,
        free_family_root=free_family_root,
        source_root=source_root,
        public_root=public_root,
        output_root=output_root,
    )


def is_forbidden_relative_path(relative_path: Path) -> bool:
    if relative_path in CARRIER_ONLY_EXCLUDED_RELATIVE_PATHS:
        return True
    if any(part in FORBIDDEN_PARTS for part in relative_path.parts):
        return True
    if any(part in FORBIDDEN_NAMES for part in relative_path.parts):
        return True
    if relative_path.suffix in FORBIDDEN_SUFFIXES:
        return True
    return False


def is_allowed_tracked_relative_path(relative_path: Path) -> bool:
    if relative_path == Path("README.md"):
        return True
    if relative_path.parts[:1] == ("docs",):
        return True
    if relative_path in TRACKED_SCRIPT_RELATIVE_PATHS:
        return True
    if relative_path in {Path("free/v0.1/0/Cargo.toml"), Path("free/v0.1/0/Cargo.lock")}:
        return True
    if relative_path.parts[:5] == ("free", "v0.1", "0", "Adapter", "v0.1"):
        return True
    if relative_path.parts[:5] == ("free", "v0.1", "0", "CRANE-Kernel", "v0.1"):
        return True
    if relative_path.parts[:4] == ("free", "v0.1", "0", "crates"):
        return True
    if relative_path.parts[:4] == ("free", "v0.1", "0", "resources"):
        return True
    if relative_path.parts[:4] == ("free", "v0.1", "0", "scripts") and relative_path.suffix in {
        ".sh",
        ".py",
    } and len(relative_path.parts) == 5:
        return True
    if relative_path.parts[:4] == ("free", "v0.1", "dev-docs", "summary"):
        return True
    if relative_path in TRACKED_DEV_DOC_EXACT_RELATIVE_PATHS:
        return True
    return False


def iter_snapshot_source_files(public_root: Path) -> list[Path]:
    files: list[Path] = []
    for path in public_root.rglob("*"):
        if path.is_symlink():
            raise fail(f"symlink source is forbidden: {path}")
        if path.is_dir():
            continue
        if not path.is_file():
            raise fail(f"non-regular source is forbidden: {path}")
        relative_path = path.relative_to(public_root)
        if is_forbidden_relative_path(relative_path):
            continue
        if not is_allowed_tracked_relative_path(relative_path):
            raise fail(f"unexpected tracked path outside exact manifest: {relative_path}")
        files.append(path)
    return sorted(files)


def copy_preserving_mode(source: Path, destination: Path) -> None:
    if not source.is_file() or source.is_symlink():
        raise fail(f"invalid source file: {source}")
    destination.parent.mkdir(parents=True, exist_ok=True)
    shutil.copy2(source, destination)
    destination.chmod(stat.S_IMODE(source.stat().st_mode))


def resolve_git_blob_mode(path: Path) -> str:
    if not path.is_file() or path.is_symlink():
        raise fail(f"invalid blob mode source: {path}")
    mode_bits = stat.S_IMODE(path.stat().st_mode)
    if mode_bits & stat.S_IXUSR:
        return "100755"
    return "100644"


def collect_snapshot_state(snapshot_root: Path) -> tuple[list[SnapshotBlob], set[str], set[str]]:
    blobs: list[SnapshotBlob] = []
    directories: set[str] = set()
    top_level_entries: set[str] = set()

    for path in sorted(snapshot_root.rglob("*")):
        relative_path = path.relative_to(snapshot_root)
        if path.is_symlink():
            raise fail(f"symlink snapshot entry is forbidden: {path}")
        top_level_entries.add(relative_path.parts[0])
        if path.is_dir():
            directories.add(relative_path.as_posix())
            continue
        if not path.is_file():
            raise fail(f"non-regular snapshot entry is forbidden: {path}")
        blobs.append(
            SnapshotBlob(
                path=relative_path.as_posix(),
                mode=resolve_git_blob_mode(path),
                data=path.read_bytes(),
            )
        )

    return blobs, directories, top_level_entries


def assert_snapshot_contract(snapshot_root: Path) -> tuple[list[SnapshotBlob], set[str], set[str]]:
    blobs, directories, top_level_entries = collect_snapshot_state(snapshot_root)
    if top_level_entries != TRACKED_TOP_LEVEL_ENTRY_SET:
        raise fail(
            f"tracked top-level entry set mismatch: {sorted(top_level_entries)} != {sorted(TRACKED_TOP_LEVEL_ENTRY_SET)}"
        )

    blob_path_set = {Path(blob.path) for blob in blobs}
    unexpected_tracked = sorted(
        relative_path.as_posix()
        for relative_path in blob_path_set
        if not is_allowed_tracked_relative_path(relative_path)
    )
    if unexpected_tracked:
        raise fail(f"unexpected tracked paths present in snapshot: {unexpected_tracked}")
    missing_required = sorted(
        required_path.as_posix()
        for required_path in REQUIRED_RELATIVE_PATHS
        if required_path not in blob_path_set
    )
    if missing_required:
        raise fail(f"missing required tracked paths: {missing_required}")

    return blobs, directories, top_level_entries


def materialize_tracked_publication_branch(roots: Roots) -> tuple[Path, list[SnapshotBlob], set[str], set[str]]:
    if roots.output_root.exists():
        shutil.rmtree(roots.output_root)
    roots.output_root.mkdir(parents=True, exist_ok=True)

    for source_path in iter_snapshot_source_files(roots.public_root):
        relative_path = source_path.relative_to(roots.public_root)
        copy_preserving_mode(source_path, roots.output_root / relative_path)

    blobs, directories, top_level_entries = assert_snapshot_contract(roots.output_root)
    return roots.output_root, blobs, directories, top_level_entries


def main() -> None:
    if len(sys.argv) != 1:
        raise fail("positional argument is forbidden")

    roots = resolve_roots()
    snapshot_root, blobs, _, top_level_entries = materialize_tracked_publication_branch(roots)
    print(
        json.dumps(
            {
                "snapshot_root": str(snapshot_root),
                "tracked_top_level_entries": sorted(top_level_entries),
                "tracked_file_count": len(blobs),
            },
            ensure_ascii=True,
        )
    )


if __name__ == "__main__":
    main()
