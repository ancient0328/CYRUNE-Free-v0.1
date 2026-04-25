#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PUBLIC_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
FREE_ROOT="$PUBLIC_ROOT/free/v0.1/0"
STATE_ROOT="$FREE_ROOT/target/public-run"
CYRUNE_HOME="$STATE_ROOT/home"
CARRIER_URL="https://github.com/ancient0328/CYRUNE/releases/download/v0.1.0/cyrune-free-v0.1.tar.gz"
CARRIER_FILENAME="cyrune-free-v0.1.tar.gz"
CARRIER_SIZE_BYTES="563981721"
CARRIER_SHA256="b358bc3517cd2c70608193b37806663cb2167eecb44532caca208a0534ae32e8"
CARRIER_ARCHIVE="$STATE_ROOT/$CARRIER_FILENAME"
CARRIER_EXTRACT_ROOT="$STATE_ROOT/carrier"
CARRIER_PACKAGE_ROOT="$CARRIER_EXTRACT_ROOT/${CARRIER_FILENAME%.tar.gz}"
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

python3 - "$CARRIER_ARCHIVE" "${CARRIER_FILENAME%.tar.gz}" <<'PY'
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
            raise SystemExit(f"absolute path member is forbidden: {member.name}")
        if any(part == ".." for part in path.parts):
            raise SystemExit(f"parent traversal member is forbidden: {member.name}")
        if member.issym():
            raise SystemExit(f"symlink member is forbidden: {member.name}")
        if member.islnk():
            raise SystemExit(f"hardlink member is forbidden: {member.name}")
        if member.ischr() or member.isblk() or member.isfifo():
            raise SystemExit(f"device file member is forbidden: {member.name}")
        if not path.parts or path.parts[0] != expected_root:
            raise SystemExit(f"unexpected top-level entry is forbidden: {member.name}")
        if path.name == "RELEASE_MANIFEST.json":
            if path != expected_manifest:
                raise SystemExit(f"manifest-outside member is forbidden: {member.name}")
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
