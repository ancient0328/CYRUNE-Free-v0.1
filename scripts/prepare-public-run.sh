#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PUBLIC_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
FREE_ROOT="$PUBLIC_ROOT/free/v0.1/0"
STATE_ROOT="$FREE_ROOT/target/public-run"
CYRUNE_HOME="$STATE_ROOT/home"
CARRIER_URL="https://github.com/ancient0328/CYRUNE-Free-v0.1/releases/download/v0.1/cyrune-free-v0.1.tar.gz"
CARRIER_ARCHIVE="$STATE_ROOT/cyrune-free-v0.1.tar.gz"
CARRIER_EXTRACT_ROOT="$STATE_ROOT/carrier"
CARRIER_PACKAGE_ROOT="$CARRIER_EXTRACT_ROOT/cyrune-free-v0.1"
CARRIER_HOME_TEMPLATE="$CARRIER_PACKAGE_ROOT/share/cyrune/home-template"
CARRIER_BUNDLE_MODEL="$CARRIER_PACKAGE_ROOT/share/cyrune/bundle-root/embedding/artifacts/multilingual-e5-small/model.onnx"
CARRIER_HOME_MODEL="$CARRIER_HOME_TEMPLATE/embedding/artifacts/multilingual-e5-small/model.onnx"

cd "$FREE_ROOT"
rm -rf "$STATE_ROOT"
install -d "$STATE_ROOT/bin" "$STATE_ROOT/home" "$CARRIER_EXTRACT_ROOT"
curl --fail --silent --show-error --location "$CARRIER_URL" --output "$CARRIER_ARCHIVE"
tar -xzf "$CARRIER_ARCHIVE" -C "$CARRIER_EXTRACT_ROOT"
test -f "$CARRIER_BUNDLE_MODEL"
test -f "$CARRIER_HOME_MODEL"
cp -R "$CARRIER_HOME_TEMPLATE"/. "$STATE_ROOT/home/"
cargo build --quiet --release --manifest-path "$FREE_ROOT/Cargo.toml" --bin cyr --bin cyrune-daemon
install -m 0755 "$FREE_ROOT/target/release/cyr" "$STATE_ROOT/bin/cyr"
install -m 0755 "$FREE_ROOT/target/release/cyrune-daemon" "$STATE_ROOT/bin/cyrune-daemon"
