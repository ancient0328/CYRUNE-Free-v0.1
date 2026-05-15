#!/bin/sh
set -eu

mode="d7"
if [ "$#" -gt 1 ]; then
  exit 10
fi
if [ "$#" -eq 1 ]; then
  mode="$1"
fi
case "$mode" in
  d7|rc1-b|rc1-c) ;;
  *) exit 10 ;;
esac

if ! script_dir=$(CDPATH= cd -- "$(dirname -- "$0")" 2>/dev/null && pwd); then
  exit 11
fi
if ! workspace_root=$(CDPATH= cd -- "$script_dir/.." 2>/dev/null && pwd); then
  exit 11
fi

stage_script="$script_dir/stage_shipping_readiness.py"
shipping_root="$workspace_root/target/shipping/S2/cyrune-free-v0.1"

if ! command -v python3 >/dev/null 2>&1; then
  exit 16
fi

if [ ! -d "$shipping_root" ] \
  || [ ! -f "$shipping_root/RELEASE_MANIFEST.json" ] \
  || { [ "$mode" = "rc1-b" ] && [ ! -f "$shipping_root/RELEASE_PREPARATION.json" ]; } \
  || { [ "$mode" = "rc1-c" ] && [ ! -f "$shipping_root/RELEASE_PREPARATION.json" ]; }; then
  python3 "$stage_script" >/dev/null
fi

if [ ! -d "$shipping_root" ] \
  || [ ! -f "$shipping_root/RELEASE_MANIFEST.json" ] \
  || { [ "$mode" = "rc1-b" ] && [ ! -f "$shipping_root/RELEASE_PREPARATION.json" ]; } \
  || { [ "$mode" = "rc1-c" ] && [ ! -f "$shipping_root/RELEASE_PREPARATION.json" ]; }; then
  exit 12
fi

if ! cargo build --quiet --manifest-path "$workspace_root/Cargo.toml" --bin d7-proof-driver >/dev/null 2>/dev/null; then
  exit 17
fi

proof_driver_bin="$workspace_root/target/debug/d7-proof-driver"
if [ ! -x "$proof_driver_bin" ]; then
  exit 17
fi

if [ "$mode" = "d7" ]; then
  artifact_root="$workspace_root/target/terminal-front-expansion/proof/D7"
  accepted_root="$artifact_root/accepted"
  fail_closed_root="$artifact_root/fail-closed"

  accepted_manifest_file="$accepted_root/d7-c3-release-manifest.json"
  accepted_snapshot_file="$accepted_root/d7-c3-productization-validation.json"
  missing_manifest_file="$fail_closed_root/d7-c3-missing-manifest.json"
  missing_notice_file="$fail_closed_root/d7-c3-missing-notice.json"
  upstream_drift_file="$fail_closed_root/d7-c3-upstream-drift.json"

  if ! mkdir -p "$accepted_root" "$fail_closed_root"; then
    exit 12
  fi

  dist_ok="$artifact_root/distribution-ok"
  dist_missing_manifest="$artifact_root/distribution-missing-manifest"
  dist_missing_notice="$artifact_root/distribution-missing-notice"
  dist_upstream_drift="$artifact_root/distribution-upstream-drift"

  rm -rf "$dist_ok" "$dist_missing_manifest" "$dist_missing_notice" "$dist_upstream_drift"
  cp -R "$shipping_root" "$dist_ok"
  cp -R "$shipping_root" "$dist_missing_manifest"
  cp -R "$shipping_root" "$dist_missing_notice"
  cp -R "$shipping_root" "$dist_upstream_drift"

  rm -f "$dist_missing_manifest/RELEASE_MANIFEST.json"
  rm -f "$dist_missing_notice/share/licenses/THIRD-PARTY-NOTICES.md"

  python3 - "$dist_upstream_drift/RELEASE_MANIFEST.json" <<'PY'
import json
import sys

path = sys.argv[1]
with open(path, encoding="utf-8") as handle:
    payload = json.load(handle)

payload["upstream_follow_triggers"] = ["security", "optional_feature"]

with open(path, "w", encoding="utf-8") as handle:
    json.dump(payload, handle, indent=2)
    handle.write("\n")
PY

  cp "$dist_ok/RELEASE_MANIFEST.json" "$accepted_manifest_file"

  "$proof_driver_bin" validate --distribution-root "$dist_ok" >"$accepted_snapshot_file"

  set +e
  "$proof_driver_bin" validate --distribution-root "$dist_missing_manifest" >"$missing_manifest_file"
  status=$?
  set -e
  if [ "$status" -ne 1 ]; then
    exit 18
  fi

  set +e
  "$proof_driver_bin" validate --distribution-root "$dist_missing_notice" >"$missing_notice_file"
  status=$?
  set -e
  if [ "$status" -ne 1 ]; then
    exit 18
  fi

  set +e
  "$proof_driver_bin" validate --distribution-root "$dist_upstream_drift" >"$upstream_drift_file"
  status=$?
  set -e
  if [ "$status" -ne 1 ]; then
    exit 18
  fi

  python3 - \
    "$accepted_manifest_file" \
    "$accepted_snapshot_file" \
    "$missing_manifest_file" \
    "$missing_notice_file" \
    "$upstream_drift_file" \
    "$dist_missing_manifest" \
    "$dist_missing_notice" \
    "$dist_upstream_drift" <<'PY'
import json
import pathlib
import sys

accepted_manifest = pathlib.Path(sys.argv[1])
accepted_snapshot = pathlib.Path(sys.argv[2])
missing_manifest = pathlib.Path(sys.argv[3])
missing_notice = pathlib.Path(sys.argv[4])
upstream_drift = pathlib.Path(sys.argv[5])
dist_missing_manifest = pathlib.Path(sys.argv[6])
dist_missing_notice = pathlib.Path(sys.argv[7])
dist_upstream_drift = pathlib.Path(sys.argv[8])

for path in [
    accepted_manifest,
    accepted_snapshot,
    missing_manifest,
    missing_notice,
    upstream_drift,
]:
    if not path.is_file():
        raise SystemExit(f"missing artifact: {path.name}")

manifest = json.loads(accepted_manifest.read_text(encoding="utf-8"))
if "productization_identity" not in manifest:
    raise SystemExit("accepted manifest missing productization_identity")

accepted_payload = json.loads(accepted_snapshot.read_text(encoding="utf-8"))
if accepted_payload.get("status") != "validated":
    raise SystemExit("accepted snapshot must be validated")
snapshot = accepted_payload.get("snapshot")
if not isinstance(snapshot, dict):
    raise SystemExit("accepted snapshot payload must be an object")
identity = snapshot.get("identity")
if identity != {
    "product_line_label": "CYRUNE Terminal",
    "packaged_product_display_name": "CYRUNE",
    "app_bundle_basename": "CYRUNE.app",
    "terminal_bundle_executable_stem": "cyrune",
}:
    raise SystemExit("accepted identity snapshot mismatch")

def load_failed(path: pathlib.Path, expected_reason: str, expected_message: str) -> None:
    payload = json.loads(path.read_text(encoding="utf-8"))
    if payload.get("status") != "failed":
        raise SystemExit(f"{path.name} must fail")
    if payload.get("surface") != "productization_failure":
        raise SystemExit(f"{path.name} surface mismatch")
    if payload.get("reason") != expected_reason:
        raise SystemExit(f"{path.name} reason mismatch")
    if payload.get("message") != expected_message:
        raise SystemExit(f"{path.name} message mismatch")
    text = path.read_text(encoding="utf-8")
    for forbidden in [
        "No such file",
        str(dist_missing_manifest),
        str(dist_missing_notice),
        str(dist_upstream_drift),
        "optional_feature",
    ]:
        if forbidden in text:
            raise SystemExit(f"{path.name} leaked forbidden detail: {forbidden}")

load_failed(
    missing_manifest,
    "productization_metadata_invalid",
    "packaged productization metadata is invalid",
)
load_failed(
    missing_notice,
    "notice_bundle_invalid",
    "packaged notice bundle is invalid",
)
load_failed(
    upstream_drift,
    "upstream_intake_judgment_invalid",
    "packaged upstream intake judgment is invalid",
)
PY

  printf 'accepted_status=validated\n'
  printf 'missing_manifest_surface=productization_failure\n'
  printf 'missing_notice_surface=productization_failure\n'
  printf 'upstream_drift_surface=productization_failure\n'
  printf 'no_raw_detail_leakage=true\n'
  exit 0
fi

if [ "$mode" = "rc1-c" ]; then
  artifact_root="$workspace_root/target/terminal-front-expansion/proof/D7-RC1"
  accepted_root="$artifact_root/accepted"
  fail_closed_root="$artifact_root/fail-closed"

  accepted_preparation_file="$accepted_root/d7-rc1-c-release-preparation.json"
  accepted_snapshot_file="$accepted_root/d7-rc1-c-organization-owned-validation.json"
  missing_signing_identity_file="$fail_closed_root/d7-rc1-c-missing-signing-identity.json"
  invalid_notarization_provider_file="$fail_closed_root/d7-rc1-c-invalid-notarization-provider.json"
  invalid_metadata_root_file="$fail_closed_root/d7-rc1-c-invalid-release-preparation-root.json"

  if ! mkdir -p "$accepted_root" "$fail_closed_root"; then
    exit 12
  fi

  dist_ok="$artifact_root/distribution-org-owned-ok"
  dist_missing_signing_identity="$artifact_root/distribution-org-owned-missing-signing-identity"
  dist_invalid_notarization_provider="$artifact_root/distribution-org-owned-invalid-notarization-provider"
  dist_invalid_metadata_root="$artifact_root/distribution-org-owned-invalid-root"

  rm -rf \
    "$dist_ok" \
    "$dist_missing_signing_identity" \
    "$dist_invalid_notarization_provider" \
    "$dist_invalid_metadata_root"
  cp -R "$shipping_root" "$dist_ok"
  cp -R "$shipping_root" "$dist_missing_signing_identity"
  cp -R "$shipping_root" "$dist_invalid_notarization_provider"
  cp -R "$shipping_root" "$dist_invalid_metadata_root"

  python3 - "$dist_ok/RELEASE_PREPARATION.json" <<'PY'
import json
import sys

path = sys.argv[1]
with open(path, encoding="utf-8") as handle:
    payload = json.load(handle)

payload["signing_identity"] = "ORG_OWNED_SIGNING_IDENTITY_FIXTURE"
payload["notarization_provider"] = "ORG_OWNED_NOTARIZATION_PROVIDER_FIXTURE"

with open(path, "w", encoding="utf-8") as handle:
    json.dump(payload, handle, indent=2)
    handle.write("\n")
PY

  python3 - "$dist_missing_signing_identity/RELEASE_PREPARATION.json" <<'PY'
import json
import sys

path = sys.argv[1]
with open(path, encoding="utf-8") as handle:
    payload = json.load(handle)

payload["notarization_provider"] = "ORG_OWNED_NOTARIZATION_PROVIDER_FIXTURE"
payload.pop("signing_identity", None)

with open(path, "w", encoding="utf-8") as handle:
    json.dump(payload, handle, indent=2)
    handle.write("\n")
PY

  python3 - "$dist_invalid_notarization_provider/RELEASE_PREPARATION.json" <<'PY'
import json
import sys

path = sys.argv[1]
with open(path, encoding="utf-8") as handle:
    payload = json.load(handle)

payload["signing_identity"] = "ORG_OWNED_SIGNING_IDENTITY_FIXTURE"
payload["notarization_provider"] = {"account": "PRIVATE-NOTARY-PROVIDER"}

with open(path, "w", encoding="utf-8") as handle:
    json.dump(payload, handle, indent=2)
    handle.write("\n")
PY

  printf '[]\n' >"$dist_invalid_metadata_root/RELEASE_PREPARATION.json"

  cp "$dist_ok/RELEASE_PREPARATION.json" "$accepted_preparation_file"

  "$proof_driver_bin" validate-rc1-c --distribution-root "$dist_ok" >"$accepted_snapshot_file"

  set +e
  "$proof_driver_bin" validate-rc1-c --distribution-root "$dist_missing_signing_identity" >"$missing_signing_identity_file"
  status=$?
  set -e
  if [ "$status" -ne 1 ]; then
    exit 18
  fi

  set +e
  "$proof_driver_bin" validate-rc1-c --distribution-root "$dist_invalid_notarization_provider" >"$invalid_notarization_provider_file"
  status=$?
  set -e
  if [ "$status" -ne 1 ]; then
    exit 18
  fi

  set +e
  "$proof_driver_bin" validate-rc1-c --distribution-root "$dist_invalid_metadata_root" >"$invalid_metadata_root_file"
  status=$?
  set -e
  if [ "$status" -ne 1 ]; then
    exit 18
  fi

  python3 - \
    "$accepted_preparation_file" \
    "$accepted_snapshot_file" \
    "$missing_signing_identity_file" \
    "$invalid_notarization_provider_file" \
    "$invalid_metadata_root_file" \
    "$dist_missing_signing_identity" \
    "$dist_invalid_notarization_provider" \
    "$dist_invalid_metadata_root" <<'PY'
import json
import pathlib
import sys

accepted_preparation = pathlib.Path(sys.argv[1])
accepted_snapshot = pathlib.Path(sys.argv[2])
missing_signing_identity = pathlib.Path(sys.argv[3])
invalid_notarization_provider = pathlib.Path(sys.argv[4])
invalid_metadata_root = pathlib.Path(sys.argv[5])
dist_missing_signing_identity = pathlib.Path(sys.argv[6])
dist_invalid_notarization_provider = pathlib.Path(sys.argv[7])
dist_invalid_metadata_root = pathlib.Path(sys.argv[8])

for path in [
    accepted_preparation,
    accepted_snapshot,
    missing_signing_identity,
    invalid_notarization_provider,
    invalid_metadata_root,
]:
    if not path.is_file():
        raise SystemExit(f"missing artifact: {path.name}")

preparation = json.loads(accepted_preparation.read_text(encoding="utf-8"))
if preparation.get("signing_identity") != "ORG_OWNED_SIGNING_IDENTITY_FIXTURE":
    raise SystemExit("accepted preparation signing identity mismatch")
if preparation.get("notarization_provider") != "ORG_OWNED_NOTARIZATION_PROVIDER_FIXTURE":
    raise SystemExit("accepted preparation notarization provider mismatch")

accepted_payload = json.loads(accepted_snapshot.read_text(encoding="utf-8"))
if accepted_payload.get("status") != "validated":
    raise SystemExit("accepted snapshot must be validated")
snapshot = accepted_payload.get("snapshot")
if snapshot != {
    "signing_identity": "ORG_OWNED_SIGNING_IDENTITY_FIXTURE",
    "notarization_provider": "ORG_OWNED_NOTARIZATION_PROVIDER_FIXTURE",
}:
    raise SystemExit("accepted organization-owned snapshot mismatch")

def load_failed(path: pathlib.Path, expected_reason: str, expected_message: str, forbidden: list[str]) -> None:
    payload = json.loads(path.read_text(encoding="utf-8"))
    if payload.get("status") != "failed":
        raise SystemExit(f"{path.name} must fail")
    if payload.get("surface") != "release_preparation_failure":
        raise SystemExit(f"{path.name} surface mismatch")
    if payload.get("reason") != expected_reason:
        raise SystemExit(f"{path.name} reason mismatch")
    if payload.get("message") != expected_message:
        raise SystemExit(f"{path.name} message mismatch")
    text = path.read_text(encoding="utf-8")
    for value in forbidden:
        if value in text:
            raise SystemExit(f"{path.name} leaked forbidden detail: {value}")

load_failed(
    missing_signing_identity,
    "signing_identity_invalid",
    "packaged signing identity is invalid",
    ["No such file", "RELEASE_PREPARATION.json.signing_identity", str(dist_missing_signing_identity)],
)
load_failed(
    invalid_notarization_provider,
    "notarization_provider_invalid",
    "packaged notarization provider is invalid",
    ["No such file", "PRIVATE-NOTARY-PROVIDER", "RELEASE_PREPARATION.json.notarization_provider", str(dist_invalid_notarization_provider)],
)
load_failed(
    invalid_metadata_root,
    "release_preparation_metadata_invalid",
    "packaged release preparation metadata is invalid",
    ["No such file", str(dist_invalid_metadata_root)],
)
PY

  printf 'accepted_status=validated\n'
  printf 'signing_identity_surface=release_preparation_failure\n'
  printf 'notarization_provider_surface=release_preparation_failure\n'
  printf 'metadata_root_surface=release_preparation_failure\n'
  printf 'no_raw_detail_leakage=true\n'
  exit 0
fi

artifact_root="$workspace_root/target/terminal-front-expansion/proof/D7-RC1"
accepted_root="$artifact_root/accepted"
fail_closed_root="$artifact_root/fail-closed"

accepted_manifest_file="$accepted_root/d7-rc1-b-release-manifest.json"
accepted_preparation_file="$accepted_root/d7-rc1-b-release-preparation.json"
accepted_snapshot_file="$accepted_root/d7-rc1-b-rule-fixed-validation.json"
bad_bundle_identifier_file="$fail_closed_root/d7-rc1-b-malformed-bundle-identifier.json"
bad_artifact_naming_file="$fail_closed_root/d7-rc1-b-invalid-artifact-naming.json"
bad_upstream_pin_file="$fail_closed_root/d7-rc1-b-upstream-pin-mismatch.json"

if ! mkdir -p "$accepted_root" "$fail_closed_root"; then
  exit 12
fi

dist_ok="$artifact_root/distribution-ok"
dist_bad_bundle_identifier="$artifact_root/distribution-bad-bundle-identifier"
dist_bad_artifact_naming="$artifact_root/distribution-bad-artifact-naming"
dist_bad_upstream_pin="$artifact_root/distribution-bad-upstream-pin"

rm -rf "$dist_ok" "$dist_bad_bundle_identifier" "$dist_bad_artifact_naming" "$dist_bad_upstream_pin"
cp -R "$shipping_root" "$dist_ok"
cp -R "$shipping_root" "$dist_bad_bundle_identifier"
cp -R "$shipping_root" "$dist_bad_artifact_naming"
cp -R "$shipping_root" "$dist_bad_upstream_pin"

python3 - "$dist_bad_bundle_identifier/RELEASE_PREPARATION.json" <<'PY'
import json
import sys

path = sys.argv[1]
with open(path, encoding="utf-8") as handle:
    payload = json.load(handle)

payload["reverse_dns_bundle_identifier"] = "Terminal"

with open(path, "w", encoding="utf-8") as handle:
    json.dump(payload, handle, indent=2)
    handle.write("\n")
PY

python3 - "$dist_bad_artifact_naming/RELEASE_PREPARATION.json" <<'PY'
import json
import sys

path = sys.argv[1]
with open(path, encoding="utf-8") as handle:
    payload = json.load(handle)

payload["archive_artifact"]["emitted_name"] = "../private.tar.gz"

with open(path, "w", encoding="utf-8") as handle:
    json.dump(payload, handle, indent=2)
    handle.write("\n")
PY

python3 - "$dist_bad_upstream_pin/RELEASE_PREPARATION.json" <<'PY'
import json
import sys

path = sys.argv[1]
with open(path, encoding="utf-8") as handle:
    payload = json.load(handle)

payload["upstream_source_pin"]["upstream_follow_triggers"] = ["security", "optional_feature"]

with open(path, "w", encoding="utf-8") as handle:
    json.dump(payload, handle, indent=2)
    handle.write("\n")
PY

cp "$dist_ok/RELEASE_MANIFEST.json" "$accepted_manifest_file"
cp "$dist_ok/RELEASE_PREPARATION.json" "$accepted_preparation_file"

"$proof_driver_bin" validate-rc1-b --distribution-root "$dist_ok" >"$accepted_snapshot_file"

set +e
"$proof_driver_bin" validate-rc1-b --distribution-root "$dist_bad_bundle_identifier" >"$bad_bundle_identifier_file"
status=$?
set -e
if [ "$status" -ne 1 ]; then
  exit 18
fi

set +e
"$proof_driver_bin" validate-rc1-b --distribution-root "$dist_bad_artifact_naming" >"$bad_artifact_naming_file"
status=$?
set -e
if [ "$status" -ne 1 ]; then
  exit 18
fi

set +e
"$proof_driver_bin" validate-rc1-b --distribution-root "$dist_bad_upstream_pin" >"$bad_upstream_pin_file"
status=$?
set -e
if [ "$status" -ne 1 ]; then
  exit 18
fi

python3 - \
  "$accepted_manifest_file" \
  "$accepted_preparation_file" \
  "$accepted_snapshot_file" \
  "$bad_bundle_identifier_file" \
  "$bad_artifact_naming_file" \
  "$bad_upstream_pin_file" \
  "$dist_bad_bundle_identifier" \
  "$dist_bad_artifact_naming" \
  "$dist_bad_upstream_pin" <<'PY'
import json
import pathlib
import sys

accepted_manifest = pathlib.Path(sys.argv[1])
accepted_preparation = pathlib.Path(sys.argv[2])
accepted_snapshot = pathlib.Path(sys.argv[3])
bad_bundle_identifier = pathlib.Path(sys.argv[4])
bad_artifact_naming = pathlib.Path(sys.argv[5])
bad_upstream_pin = pathlib.Path(sys.argv[6])
dist_bad_bundle_identifier = pathlib.Path(sys.argv[7])
dist_bad_artifact_naming = pathlib.Path(sys.argv[8])
dist_bad_upstream_pin = pathlib.Path(sys.argv[9])

for path in [
    accepted_manifest,
    accepted_preparation,
    accepted_snapshot,
    bad_bundle_identifier,
    bad_artifact_naming,
    bad_upstream_pin,
]:
    if not path.is_file():
        raise SystemExit(f"missing artifact: {path.name}")

manifest = json.loads(accepted_manifest.read_text(encoding="utf-8"))
if manifest.get("distribution_unit") != "cyrune-free-v0.1.1-beta.1.tar.gz":
    raise SystemExit("accepted manifest distribution unit mismatch")

preparation = json.loads(accepted_preparation.read_text(encoding="utf-8"))
if preparation.get("reverse_dns_bundle_identifier") != "local.cyrune.terminal":
    raise SystemExit("accepted preparation bundle identifier mismatch")

accepted_payload = json.loads(accepted_snapshot.read_text(encoding="utf-8"))
if accepted_payload.get("status") != "validated":
    raise SystemExit("accepted snapshot must be validated")
snapshot = accepted_payload.get("snapshot")
if not isinstance(snapshot, dict):
    raise SystemExit("accepted snapshot payload must be an object")
if snapshot.get("reverse_dns_bundle_identifier") != "local.cyrune.terminal":
    raise SystemExit("accepted reverse-DNS bundle identifier mismatch")
if snapshot.get("installer_artifact") != {
    "artifact_class": "app_bundle",
    "platform": "macOS",
    "emitted_name": "CYRUNE.app",
}:
    raise SystemExit("accepted installer artifact mismatch")
if snapshot.get("archive_artifact") != {
    "artifact_class": "distribution_archive",
    "platform": "macOS",
    "emitted_name": "cyrune-free-v0.1.1-beta.1.tar.gz",
}:
    raise SystemExit("accepted archive artifact mismatch")
if snapshot.get("upstream_source_pin") != {
    "source_project": "wezterm/wezterm",
    "source_kind": "github-release-tag",
    "exact_revision": "20240203-110809-5046fc22",
    "source_archive": "wezterm-20240203-110809-5046fc22-src.tar.gz",
    "evidence_origin": "official-github-release",
    "source_reference_url": "https://github.com/wezterm/wezterm/releases/tag/20240203-110809-5046fc22",
    "upstream_intake_mode": "evidence-based",
    "upstream_follow_triggers": ["security", "critical_bug", "required_feature"],
    "upstream_auto_follow": False,
}:
    raise SystemExit("accepted upstream source pin mismatch")

def load_failed(path: pathlib.Path, expected_reason: str, expected_message: str, forbidden: list[str]) -> None:
    payload = json.loads(path.read_text(encoding="utf-8"))
    if payload.get("status") != "failed":
        raise SystemExit(f"{path.name} must fail")
    if payload.get("surface") != "release_preparation_failure":
        raise SystemExit(f"{path.name} surface mismatch")
    if payload.get("reason") != expected_reason:
        raise SystemExit(f"{path.name} reason mismatch")
    if payload.get("message") != expected_message:
        raise SystemExit(f"{path.name} message mismatch")
    text = path.read_text(encoding="utf-8")
    for value in forbidden:
        if value in text:
            raise SystemExit(f"{path.name} leaked forbidden detail: {value}")

load_failed(
    bad_bundle_identifier,
    "bundle_identifier_invalid",
    "packaged reverse-DNS bundle identifier is invalid",
    ["No such file", "Terminal", str(dist_bad_bundle_identifier)],
)
load_failed(
    bad_artifact_naming,
    "artifact_naming_invalid",
    "packaged release artifact naming is invalid",
    ["No such file", "../private.tar.gz", str(dist_bad_artifact_naming)],
)
load_failed(
    bad_upstream_pin,
    "upstream_source_pin_invalid",
    "packaged upstream source pin is invalid",
    ["No such file", "optional_feature", str(dist_bad_upstream_pin)],
)
PY

printf 'accepted_status=validated\n'
printf 'bundle_identifier_surface=release_preparation_failure\n'
printf 'artifact_naming_surface=release_preparation_failure\n'
printf 'upstream_pin_surface=release_preparation_failure\n'
printf 'no_raw_detail_leakage=true\n'
