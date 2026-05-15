#!/bin/sh
set -eu

if [ "$#" -ne 0 ]; then
  exit 10
fi

if ! script_dir=$(CDPATH= cd -- "$(dirname -- "$0")" 2>/dev/null && pwd); then
  exit 11
fi
if ! workspace_root=$(CDPATH= cd -- "$script_dir/.." 2>/dev/null && pwd); then
  exit 11
fi

stage_script="$script_dir/stage_shipping_readiness.py"
shipping_root="$workspace_root/target/shipping/S2/cyrune-free-v0.1"
artifact_root="$workspace_root/target/terminal-front-expansion/proof/D5"
release_manifest_file="$artifact_root/release-manifest.json"
sha256sums_file="$artifact_root/sha256sums.txt"
doctor_health_file="$artifact_root/doctor-health.json"
no_llm_response_file="$artifact_root/packaged-no-llm-response.json"
adapter_response_file="$artifact_root/packaged-adapter-response.json"
missing_binding_reject_file="$artifact_root/missing-binding-no-llm-reject.json"
home_copy_reject_file="$artifact_root/home-registry-copy-does-not-rescue-adapter-reject.json"
invalid_override_fail_file="$artifact_root/invalid-override-doctor-fail.txt"

if ! command -v python3 >/dev/null 2>&1; then
  exit 16
fi

python3 "$stage_script" >/dev/null

if [ ! -d "$shipping_root" ]; then
  exit 12
fi
if [ ! -f "$shipping_root/RELEASE_MANIFEST.json" ]; then
  exit 12
fi
if [ ! -f "$shipping_root/SHA256SUMS.txt" ]; then
  exit 12
fi

if ! rm -rf "$artifact_root"; then
  exit 12
fi
if ! mkdir -p "$artifact_root"; then
  exit 12
fi

cp "$shipping_root/RELEASE_MANIFEST.json" "$release_manifest_file"
cp "$shipping_root/SHA256SUMS.txt" "$sha256sums_file"

bundle_root_rel="$(
  python3 - "$release_manifest_file" <<'PY'
import json
import sys

with open(sys.argv[1], encoding="utf-8") as handle:
    payload = json.load(handle)

value = payload.get("bundle_root_path")
if not isinstance(value, str) or value == "":
    raise SystemExit(1)

print(value)
PY
)"

dist_ok="$artifact_root/distribution-ok"
dist_missing_binding="$artifact_root/distribution-missing-binding"
dist_missing_registry="$artifact_root/distribution-missing-registry"
home_ok="$artifact_root/home-ok"
home_home_registry_copy="$artifact_root/home-home-registry-copy"
home_invalid_override="$artifact_root/home-invalid-override"
workspace_ok="$artifact_root/workspace-ok"
workspace_home_registry_copy="$artifact_root/workspace-home-registry-copy"
cyr_ok="$dist_ok/bin/cyr"

cp -R "$shipping_root" "$dist_ok"

bundle_root_ok="$dist_ok/$bundle_root_rel"
if [ ! -d "$bundle_root_ok" ]; then
  exit 12
fi

mkdir -p "$home_ok" "$workspace_ok"
CYRUNE_DISTRIBUTION_ROOT="$dist_ok" \
CYRUNE_HOME="$home_ok" \
CRANE_ROOT="/nonexistent/should-not-be-used" \
  "$cyr_ok" doctor >"$doctor_health_file"

CYRUNE_DISTRIBUTION_ROOT="$dist_ok" \
CYRUNE_HOME="$home_ok" \
CRANE_ROOT="/nonexistent/should-not-be-used" \
  "$cyr_ok" run --no-llm --input "packaged mode no-llm proof" >"$no_llm_response_file"

CYRUNE_DISTRIBUTION_ROOT="$dist_ok" \
CYRUNE_HOME="$home_ok" \
CRANE_ROOT="/nonexistent/should-not-be-used" \
  "$cyr_ok" run --adapter local-cli-single-process.v0.1 --input "packaged mode approved adapter proof" --cap exec --cap fs_read --cwd "$workspace_ok" >"$adapter_response_file"

cp -R "$dist_ok" "$dist_missing_binding"
rm -f "$dist_missing_binding/$bundle_root_rel/adapter/bindings/cyrune-free-default.v0.1.json"
CYRUNE_DISTRIBUTION_ROOT="$dist_missing_binding" \
CYRUNE_HOME="$home_ok" \
CRANE_ROOT="/nonexistent/should-not-be-used" \
  "$dist_missing_binding/bin/cyr" run --no-llm --input "missing bundle binding proof" >"$missing_binding_reject_file"

cp -R "$dist_ok" "$dist_missing_registry"
rm -f "$dist_missing_registry/$bundle_root_rel/registry/execution-adapters/approved/registry.json"
mkdir -p "$home_home_registry_copy/registry/execution-adapters"
cp -R \
  "$bundle_root_ok/registry/execution-adapters/approved" \
  "$home_home_registry_copy/registry/execution-adapters/approved"
mkdir -p "$workspace_home_registry_copy"
CYRUNE_DISTRIBUTION_ROOT="$dist_missing_registry" \
CYRUNE_HOME="$home_home_registry_copy" \
CRANE_ROOT="/nonexistent/should-not-be-used" \
  "$dist_missing_registry/bin/cyr" run --adapter local-cli-single-process.v0.1 --input "home copy must not rescue authority proof" --cap exec --cap fs_read --cwd "$workspace_home_registry_copy" >"$home_copy_reject_file"

set +e
CYRUNE_DISTRIBUTION_ROOT="relative/path" \
CYRUNE_HOME="$home_invalid_override" \
CRANE_ROOT="/nonexistent/should-not-be-used" \
  "$cyr_ok" doctor >/dev/null 2>/dev/null
status=$?
set -e
printf 'exit=%s\n' "$status" >"$invalid_override_fail_file"

python3 - \
  "$artifact_root" \
  "$dist_ok" \
  "$bundle_root_rel" <<'PY'
import json
import pathlib
import sys

artifact_root = pathlib.Path(sys.argv[1])
dist_ok = pathlib.Path(sys.argv[2])
bundle_root_rel = pathlib.Path(sys.argv[3])
bundle_root = dist_ok / bundle_root_rel

required_files = [
    "release-manifest.json",
    "sha256sums.txt",
    "doctor-health.json",
    "packaged-no-llm-response.json",
    "packaged-adapter-response.json",
    "missing-binding-no-llm-reject.json",
    "home-registry-copy-does-not-rescue-adapter-reject.json",
    "invalid-override-doctor-fail.txt",
]
for name in required_files:
    path = artifact_root / name
    if not path.is_file():
        raise SystemExit(f"missing artifact: {name}")

manifest = json.loads((artifact_root / "release-manifest.json").read_text(encoding="utf-8"))
if not isinstance(manifest, dict):
    raise SystemExit("release-manifest.json must be a JSON object")
expected_manifest = {
    "bundle_root_path": "share/cyrune/bundle-root",
    "home_template_path": "share/cyrune/home-template",
    "runtime_entry": "bin/cyr",
    "daemon_entry": "bin/cyrune-daemon",
}
for key, expected in expected_manifest.items():
    if manifest.get(key) != expected:
        raise SystemExit(f"release-manifest.json invalid {key}")

sha256sums = (artifact_root / "sha256sums.txt").read_text(encoding="utf-8")
for entry in [
    "RELEASE_MANIFEST.json",
    "bin/cyr",
    "bin/cyrune-daemon",
]:
    if entry not in sha256sums:
        raise SystemExit(f"sha256sums.txt missing {entry}")

doctor = json.loads((artifact_root / "doctor-health.json").read_text(encoding="utf-8"))
if not isinstance(doctor, dict):
    raise SystemExit("doctor-health.json must be a JSON object")
if doctor.get("status") != "healthy":
    raise SystemExit("doctor status must be healthy")
if doctor.get("distribution_root") != str(dist_ok):
    raise SystemExit("doctor distribution_root mismatch")
if doctor.get("bundle_root") != str(bundle_root):
    raise SystemExit("doctor bundle_root mismatch")
if doctor.get("bundle_ready") is not True:
    raise SystemExit("doctor bundle_ready must be true")

required_response_fields = [
    "response_to",
    "correlation_id",
    "run_id",
    "evidence_id",
    "citation_bundle_id",
    "working_hash_after",
    "policy_pack_id",
]
for name in [
    "packaged-no-llm-response.json",
    "packaged-adapter-response.json",
]:
    payload = json.loads((artifact_root / name).read_text(encoding="utf-8"))
    if not isinstance(payload, dict):
        raise SystemExit(f"{name} must be a JSON object")
    for key in required_response_fields:
        value = payload.get(key)
        if not isinstance(value, str) or value == "":
            raise SystemExit(f"{name} missing required non-empty field: {key}")

for name in [
    "missing-binding-no-llm-reject.json",
    "home-registry-copy-does-not-rescue-adapter-reject.json",
]:
    payload = json.loads((artifact_root / name).read_text(encoding="utf-8"))
    if not isinstance(payload, dict):
        raise SystemExit(f"{name} must be a JSON object")
    if payload.get("reason_kind") != "binding_unresolved":
        raise SystemExit(f"{name} reason_kind must be binding_unresolved")
    rule_id = payload.get("rule_id")
    if not isinstance(rule_id, str) or not rule_id.startswith("BND-"):
        raise SystemExit(f"{name} rule_id must start with BND-")

invalid_override_lines = (
    artifact_root / "invalid-override-doctor-fail.txt"
).read_text(encoding="utf-8").splitlines()
if len(invalid_override_lines) != 1:
    raise SystemExit("invalid-override-doctor-fail.txt must contain exactly one line")
line = invalid_override_lines[0]
if not line.startswith("exit="):
    raise SystemExit("invalid-override-doctor-fail.txt must start with exit=")
try:
    exit_code = int(line[5:])
except ValueError as exc:
    raise SystemExit("invalid-override-doctor-fail.txt must contain an integer exit code") from exc
if exit_code == 0:
    raise SystemExit("invalid override doctor failure must be non-zero")
PY
