from __future__ import annotations

import json
import shutil
import stat
import textwrap
from pathlib import Path

REPOSITORY_OWNER = "ancient0328"
REPOSITORY_NAME = "CYRUNE"
RELEASE_TAG = "v0.1.0"
REPOSITORY_FULL_NAME = f"{REPOSITORY_OWNER}/{REPOSITORY_NAME}"

README_BODY = """# CYRUNE Free v0.1

## What This Is

CYRUNE Free v0.1 is the public single-user Control OS publication unit for the current accepted scope.

## Start Here

- [Public Index](docs/CYRUNE_Free_Public_Index.md)
- [Getting Started](docs/GETTING_STARTED.md)
- [Troubleshooting](docs/TROUBLESHOOTING.md)
- [Free Source](free/v0.1/0/)

## What You Get

- Tracked public branch surface: `README.md`, `docs/`, `scripts/`, `free/`
- GitHub-hosted non-tracked carrier: `cyrune-free-v0.1.tar.gz`
- Docs are auxiliary and do not replace the carrier

## Versioning

- `main` is the latest public repository surface.
- SemVer tags, such as `v0.1.0`, are immutable snapshots of this public repository content.
- `v0.1` is treated as a version marker / compatibility tag, not as a branch name.
- A `v0.1` branch is not used.

## Not Included

- Native distributable release
- Concrete signing / notarization values
- Private development / internal operational corpus
"""

GETTING_STARTED_BODY = """# GETTING_STARTED

Run the three scripts in order from the tracked public branch surface. `prepare-public-run.sh` downloads the exact release asset `cyrune-free-v0.1.tar.gz`, normalizes the required non-tracked carrier into `target/public-run/`, and then prepares the local runtime state. Do not skip steps or change the sequence.

## 1. prepare-public-run.sh

```bash
./scripts/prepare-public-run.sh
```

## 2. doctor.sh

```bash
./scripts/doctor.sh
```

## 3. first-success.sh

```bash
./scripts/first-success.sh
```
"""

TROUBLESHOOTING_BODY = """# TROUBLESHOOTING

These remediation notes are limited to the three public scripts and do not extend to internal or native-distribution workflows.

## prepare-public-run.sh

If this step fails, confirm the exact release asset URL is reachable, confirm carrier download and extraction succeeded, then rerun ./scripts/prepare-public-run.sh.

## doctor.sh

If this step fails, rerun ./scripts/prepare-public-run.sh first, then rerun ./scripts/doctor.sh.

## first-success.sh

If this step fails, rerun ./scripts/prepare-public-run.sh, confirm ./scripts/doctor.sh passes, then rerun ./scripts/first-success.sh.
"""

DOCTOR_BODY = """#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PUBLIC_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
FREE_ROOT="$PUBLIC_ROOT/free/v0.1/0"
STATE_ROOT="$FREE_ROOT/target/public-run"
CYRUNE_HOME="$STATE_ROOT/home"
export CYRUNE_HOME

cd "$FREE_ROOT"
"$STATE_ROOT/bin/cyr" doctor
"""

FIRST_SUCCESS_BODY = """#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PUBLIC_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
FREE_ROOT="$PUBLIC_ROOT/free/v0.1/0"
STATE_ROOT="$FREE_ROOT/target/public-run"
CYRUNE_HOME="$STATE_ROOT/home"
export CYRUNE_HOME

cd "$FREE_ROOT"
"$STATE_ROOT/bin/cyr" run --no-llm --input "ship-goal public first success"
"""


def parse_single_hash_line(path: Path, expected_filename: str) -> str:
    ensure_regular_file(path)
    lines = [line.strip() for line in path.read_text(encoding="utf-8").splitlines() if line.strip()]
    if len(lines) != 1:
        raise fail(f"expected exactly one archive hash line in {path}")
    digest, separator, filename = lines[0].partition("  ")
    if separator != "  " or not digest or filename != expected_filename:
        raise fail(f"unexpected archive hash sidecar payload: {path}")
    return digest


def derive_carrier_tuple(standalone_root: Path) -> dict[str, str]:
    shipping_root = standalone_root / "target" / "shipping" / "S2"
    package_root = shipping_root / "cyrune-free-v0.1"
    release_manifest_path = package_root / "RELEASE_MANIFEST.json"
    archive_hash_sidecar_path = shipping_root / "guard" / "archive-sha256.txt"

    ensure_regular_file(release_manifest_path)
    manifest = json.loads(release_manifest_path.read_text(encoding="utf-8"))
    carrier_filename = manifest.get("distribution_unit")
    if not isinstance(carrier_filename, str) or not carrier_filename:
        raise fail(f"distribution_unit is missing in {release_manifest_path}")
    archive_path = shipping_root / carrier_filename
    ensure_regular_file(archive_path)
    carrier_sha256 = parse_single_hash_line(archive_hash_sidecar_path, carrier_filename)
    carrier_size_bytes = str(archive_path.stat().st_size)
    carrier_url = (
        f"https://github.com/{REPOSITORY_FULL_NAME}/releases/download/"
        f"{RELEASE_TAG}/{carrier_filename}"
    )
    return {
        "url": carrier_url,
        "filename": carrier_filename,
        "size_bytes": carrier_size_bytes,
        "sha256": carrier_sha256,
    }


def render_prepare_public_run_body(carrier: dict[str, str]) -> str:
    return textwrap.dedent(
        f"""\
        #!/usr/bin/env bash
        set -euo pipefail

        SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
        PUBLIC_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
        FREE_ROOT="$PUBLIC_ROOT/free/v0.1/0"
        STATE_ROOT="$FREE_ROOT/target/public-run"
        CYRUNE_HOME="$STATE_ROOT/home"
        CARRIER_URL="{carrier["url"]}"
        CARRIER_FILENAME="{carrier["filename"]}"
        CARRIER_SIZE_BYTES="{carrier["size_bytes"]}"
        CARRIER_SHA256="{carrier["sha256"]}"
        CARRIER_ARCHIVE="$STATE_ROOT/$CARRIER_FILENAME"
        CARRIER_EXTRACT_ROOT="$STATE_ROOT/carrier"
        CARRIER_PACKAGE_ROOT="$CARRIER_EXTRACT_ROOT/${{CARRIER_FILENAME%.tar.gz}}"
        CARRIER_HOME_TEMPLATE="$CARRIER_PACKAGE_ROOT/share/cyrune/home-template"
        CARRIER_BUNDLE_MODEL="$CARRIER_PACKAGE_ROOT/share/cyrune/bundle-root/embedding/artifacts/multilingual-e5-small/model.onnx"
        CARRIER_HOME_MODEL="$CARRIER_HOME_TEMPLATE/embedding/artifacts/multilingual-e5-small/model.onnx"

        cd "$FREE_ROOT"
        rm -rf "$STATE_ROOT"
        install -d "$STATE_ROOT/bin" "$STATE_ROOT/home" "$CARRIER_EXTRACT_ROOT"
        curl --fail --silent --show-error --location "$CARRIER_URL" --output "$CARRIER_ARCHIVE"

        ACTUAL_FILENAME="$(basename "$CARRIER_ARCHIVE")"
        if [ "$ACTUAL_FILENAME" != "$CARRIER_FILENAME" ]; then
          echo "carrier filename mismatch: $ACTUAL_FILENAME" >&2
          exit 1
        fi

        ACTUAL_SIZE_BYTES="$(python3 - "$CARRIER_ARCHIVE" <<'PY'
        import os
        import sys

        print(os.path.getsize(sys.argv[1]))
        PY
        )"
        if [ "$ACTUAL_SIZE_BYTES" != "$CARRIER_SIZE_BYTES" ]; then
          echo "carrier size mismatch: $ACTUAL_SIZE_BYTES" >&2
          exit 1
        fi

        ACTUAL_SHA256="$(python3 - "$CARRIER_ARCHIVE" <<'PY'
        import hashlib
        import sys
        from pathlib import Path

        digest = hashlib.sha256()
        with Path(sys.argv[1]).open("rb") as handle:
            for chunk in iter(lambda: handle.read(65536), b""):
                digest.update(chunk)
        print(digest.hexdigest())
        PY
        )"
        if [ "$ACTUAL_SHA256" != "$CARRIER_SHA256" ]; then
          echo "carrier sha256 mismatch: $ACTUAL_SHA256" >&2
          exit 1
        fi

        python3 - "$CARRIER_ARCHIVE" "${{CARRIER_FILENAME%.tar.gz}}" <<'PY'
        import pathlib
        import sys
        import tarfile

        archive_path = pathlib.Path(sys.argv[1])
        expected_root = sys.argv[2]
        expected_manifest = pathlib.PurePosixPath(expected_root, "RELEASE_MANIFEST.json")
        has_manifest = False

        with tarfile.open(archive_path, "r:gz") as archive:
            for member in archive.getmembers():
                if not member.name:
                    raise SystemExit("empty archive member is forbidden")

                path = pathlib.PurePosixPath(member.name)
                if path.is_absolute():
                    raise SystemExit(f"absolute path member is forbidden: {{member.name}}")
                if any(part == ".." for part in path.parts):
                    raise SystemExit(f"parent traversal member is forbidden: {{member.name}}")
                if member.issym():
                    raise SystemExit(f"symlink member is forbidden: {{member.name}}")
                if member.islnk():
                    raise SystemExit(f"hardlink member is forbidden: {{member.name}}")
                if member.ischr() or member.isblk() or member.isfifo():
                    raise SystemExit(f"device file member is forbidden: {{member.name}}")
                if not path.parts or path.parts[0] != expected_root:
                    raise SystemExit(f"unexpected top-level entry is forbidden: {{member.name}}")
                if path.name == "RELEASE_MANIFEST.json":
                    if path != expected_manifest:
                        raise SystemExit(f"manifest-outside member is forbidden: {{member.name}}")
                    has_manifest = True

        if not has_manifest:
            raise SystemExit("missing expected release manifest inside carrier archive")
        PY

        tar -xzf "$CARRIER_ARCHIVE" -C "$CARRIER_EXTRACT_ROOT"
        test -f "$CARRIER_BUNDLE_MODEL"
        test -f "$CARRIER_HOME_MODEL"
        cp -R "$CARRIER_HOME_TEMPLATE"/. "$STATE_ROOT/home/"
        cargo build --quiet --release --manifest-path "$FREE_ROOT/Cargo.toml" --bin cyrune-runtime-cli --bin cyrune-daemon
        install -m 0755 "$FREE_ROOT/target/release/cyrune-runtime-cli" "$STATE_ROOT/bin/cyr"
        install -m 0755 "$FREE_ROOT/target/release/cyrune-daemon" "$STATE_ROOT/bin/cyrune-daemon"
        """
    )

DOC_FILE_MAPPINGS: tuple[tuple[str, str], ...] = (
    (
        "free/v0.1/dev-docs/00-TARGET_SYSTEM.md",
        "free/v0.1/dev-docs/00-TARGET_SYSTEM.md",
    ),
    (
        "free/v0.1/dev-docs/03-architecture/ARCHITECTURE_OVERVIEW.md",
        "free/v0.1/dev-docs/03-architecture/ARCHITECTURE_OVERVIEW.md",
    ),
    (
        "free/v0.1/dev-docs/90-reports/20260410-terminal-D6-native-outer-launcher-proof.md",
        "free/v0.1/dev-docs/90-reports/20260410-terminal-D6-native-outer-launcher-proof.md",
    ),
    (
        "free/v0.1/dev-docs/90-reports/20260411-terminal-D7-terminal-bundle-productization-proof.md",
        "free/v0.1/dev-docs/90-reports/20260411-terminal-D7-terminal-bundle-productization-proof.md",
    ),
    (
        "free/v0.1/dev-docs/90-reports/20260412-terminal-EVID-D7RC1D-1-external-release-concretization-closeout.md",
        "free/v0.1/dev-docs/90-reports/20260412-terminal-EVID-D7RC1D-1-external-release-concretization-closeout.md",
    ),
)

TREE_MAPPINGS: tuple[tuple[str, str], ...] = (
    ("docs", "docs"),
    ("free/v0.1/0", "free/v0.1/0"),
    ("free/v0.1/dev-docs/summary", "free/v0.1/dev-docs/summary"),
)

NEW_TEXT_FILES: tuple[tuple[str, str, int], ...] = (
    ("README.md", README_BODY, 0o644),
    ("docs/GETTING_STARTED.md", GETTING_STARTED_BODY, 0o644),
    ("docs/TROUBLESHOOTING.md", TROUBLESHOOTING_BODY, 0o644),
    ("scripts/doctor.sh", DOCTOR_BODY, 0o755),
    ("scripts/first-success.sh", FIRST_SUCCESS_BODY, 0o755),
)

EXCLUDED_PARTS = {"target", "__pycache__"}
EXCLUDED_NAMES = {".DS_Store"}
EXCLUDED_SUFFIXES = {".pyc"}
CARRIER_ONLY_EXCLUDED_FREE_ROOT_PATHS = {
    Path("resources/bundle-root/embedding/artifacts/multilingual-e5-small/model.onnx")
}


def fail(message: str) -> RuntimeError:
    return RuntimeError(message)


def is_excluded_source_path(path: Path) -> bool:
    if any(part in EXCLUDED_PARTS for part in path.parts):
        return True
    if path.name in EXCLUDED_NAMES:
        return True
    if path.suffix in EXCLUDED_SUFFIXES:
        return True
    return False


def is_carrier_only_excluded_free_root_path(path: Path) -> bool:
    return path in CARRIER_ONLY_EXCLUDED_FREE_ROOT_PATHS


def ensure_regular_file(path: Path) -> None:
    if not path.exists():
        raise fail(f"missing source path: {path}")
    if path.is_symlink():
        raise fail(f"symlink source is forbidden: {path}")
    if not path.is_file():
        raise fail(f"non-regular source is forbidden: {path}")


def iter_recursive_files(root: Path) -> list[Path]:
    if not root.exists():
        raise fail(f"missing source tree: {root}")
    if root.is_symlink():
        raise fail(f"symlink tree root is forbidden: {root}")
    if not root.is_dir():
        raise fail(f"non-directory tree root is forbidden: {root}")

    files: list[Path] = []
    for path in root.rglob("*"):
        if is_excluded_source_path(path.relative_to(root)):
            continue
        if path.is_symlink():
            raise fail(f"symlink source is forbidden: {path}")
        if path.is_dir():
            continue
        if not path.is_file():
            raise fail(f"non-regular source is forbidden: {path}")
        files.append(path)
    return sorted(files)


def copy_preserving_mode(source: Path, destination: Path) -> None:
    ensure_regular_file(source)
    destination.parent.mkdir(parents=True, exist_ok=True)
    shutil.copy2(source, destination)
    destination.chmod(stat.S_IMODE(source.stat().st_mode))


def write_text_file(destination: Path, body: str, mode: int) -> None:
    destination.parent.mkdir(parents=True, exist_ok=True)
    destination.write_text(body, encoding="utf-8", newline="\n")
    destination.chmod(mode)


def main() -> None:
    script_path = Path(__file__).resolve()
    script_dir = script_path.parent
    standalone_root = script_dir.parent
    version_root = standalone_root.parent
    free_family_root = version_root.parent
    source_root = free_family_root.parent

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

    carrier = derive_carrier_tuple(standalone_root)

    public_root = source_root / "public" / "free-v0.1"
    if public_root.exists():
        shutil.rmtree(public_root)
    public_root.mkdir(parents=True, exist_ok=True)

    for source_rel, destination_rel in DOC_FILE_MAPPINGS:
        copy_preserving_mode(source_root / source_rel, public_root / destination_rel)

    for source_rel, destination_rel in TREE_MAPPINGS:
        source_tree = source_root / source_rel
        destination_tree = public_root / destination_rel
        for source_path in iter_recursive_files(source_tree):
            relative_path = source_path.relative_to(source_tree)
            if source_rel == "free/v0.1/0" and is_carrier_only_excluded_free_root_path(
                relative_path
            ):
                continue
            copy_preserving_mode(source_path, destination_tree / relative_path)

    for destination_rel, body, mode in NEW_TEXT_FILES:
        write_text_file(public_root / destination_rel, body, mode)
    write_text_file(
        public_root / "scripts" / "prepare-public-run.sh",
        render_prepare_public_run_body(carrier),
        0o755,
    )


if __name__ == "__main__":
    main()
