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
artifact_root="$workspace_root/target/shipping-memory/SM-D/SMD-T1"
response_file="$artifact_root/packaged-shipping-response.json"
policy_file="$artifact_root/packaged-shipping-policy.json"
materialized_file="$artifact_root/materialized-paths.txt"

if ! command -v python3 >/dev/null 2>&1; then
  exit 16
fi

python3 "$stage_script" >/dev/null

if [ ! -d "$shipping_root" ]; then
  exit 12
fi

if ! rm -rf "$artifact_root"; then
  exit 12
fi
if ! mkdir -p "$artifact_root"; then
  exit 12
fi

dist_ok="$artifact_root/distribution-shipping"
home_ok="$artifact_root/home"
cp -R "$shipping_root" "$dist_ok"

cyr_bin="$dist_ok/bin/cyr"
if [ ! -x "$cyr_bin" ]; then
  exit 12
fi

CYRUNE_DISTRIBUTION_ROOT="$dist_ok" \
CYRUNE_HOME="$home_ok" \
CRANE_ROOT="/nonexistent/should-not-be-used" \
  "$cyr_bin" run --no-llm --binding cyrune-free-shipping.v0.1 --input "packaged shipping binding proof" >"$response_file"

python3 - "$response_file" "$home_ok" "$policy_file" "$materialized_file" <<'PY'
import json
import pathlib
import sys

response_path = pathlib.Path(sys.argv[1])
home_root = pathlib.Path(sys.argv[2])
policy_copy_path = pathlib.Path(sys.argv[3])
materialized_path = pathlib.Path(sys.argv[4])

payload = json.loads(response_path.read_text(encoding="utf-8"))
required_response_fields = [
    "response_to",
    "correlation_id",
    "run_id",
    "evidence_id",
    "citation_bundle_id",
    "working_hash_after",
    "policy_pack_id",
]
for key in required_response_fields:
    value = payload.get(key)
    if not isinstance(value, str) or value == "":
        raise SystemExit(f"response missing required non-empty field: {key}")

if payload["policy_pack_id"] != "cyrune-free-default":
    raise SystemExit("policy_pack_id must stay cyrune-free-default")

evidence_dir = home_root / "ledger" / "evidence" / payload["evidence_id"]
policy_path = evidence_dir / "policy.json"
if not policy_path.is_file():
    raise SystemExit("policy.json missing")

policy = json.loads(policy_path.read_text(encoding="utf-8"))
expected_roots = {
    "processing_state_root": str(home_root / "memory" / "processing"),
    "permanent_state_root": str(home_root / "memory" / "permanent"),
}
checks = [
    (policy.get("binding_id") == "cyrune-free-shipping.v0.1", "binding_id mismatch"),
    (
        policy["resolved_kernel_adapters"]["processing_store_adapter_id"] == "memory-redb-processing",
        "processing adapter mismatch",
    ),
    (
        policy["resolved_kernel_adapters"]["permanent_store_adapter_id"] == "memory-stoolap-permanent",
        "permanent adapter mismatch",
    ),
    (
        policy["memory_state_roots"]["processing_state_root"] == expected_roots["processing_state_root"],
        "processing state root mismatch",
    ),
    (
        policy["memory_state_roots"]["permanent_state_root"] == expected_roots["permanent_state_root"],
        "permanent state root mismatch",
    ),
]
for ok, message in checks:
    if not ok:
        raise SystemExit(message)

processing_db = home_root / "memory" / "processing" / "processing.redb"
permanent_root = home_root / "memory" / "permanent"
if not processing_db.is_file():
    raise SystemExit("processing redb file missing")
if not permanent_root.is_dir():
    raise SystemExit("permanent state root missing")

policy_copy_path.write_text(json.dumps(policy, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
materialized_path.write_text(
    "\n".join(
        [
            f"evidence_id={payload['evidence_id']}",
            f"correlation_id={payload['correlation_id']}",
            f"processing_state_root={expected_roots['processing_state_root']}",
            f"permanent_state_root={expected_roots['permanent_state_root']}",
            f"processing_db={processing_db}",
            f"permanent_root={permanent_root}",
        ]
    )
    + "\n",
    encoding="utf-8",
)
PY
