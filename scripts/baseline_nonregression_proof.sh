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

artifact_root="$workspace_root/target/shipping-memory/SM-D/SMD-T2"
packaged_run_log="$artifact_root/packaged-mode-smoke.txt"
dev_front_run_log="$artifact_root/dev-front-smoke.txt"
source_root="$workspace_root/target/terminal-front-expansion/proof/D5"
copied_root="$artifact_root/d5-proof"
dev_front_source_root="$workspace_root/target/developer-demo-front/proof/D3"
dev_front_copied_root="$artifact_root/dev-front-proof"

if ! rm -rf "$artifact_root"; then
  exit 12
fi
if ! mkdir -p "$artifact_root"; then
  exit 12
fi

sh "$script_dir/packaged_mode_smoke.sh" >"$packaged_run_log" 2>&1
sh "$script_dir/dev_front_smoke.sh" >"$dev_front_run_log" 2>&1

if [ ! -d "$source_root" ]; then
  exit 13
fi
if [ ! -d "$dev_front_source_root" ]; then
  exit 13
fi
cp -R "$source_root" "$copied_root"
cp -R "$dev_front_source_root" "$dev_front_copied_root"

for required in \
  "doctor-health.json" \
  "packaged-no-llm-response.json" \
  "packaged-adapter-response.json" \
  "missing-binding-no-llm-reject.json" \
  "home-registry-copy-does-not-rescue-adapter-reject.json" \
  "invalid-override-doctor-fail.txt"
do
  if [ ! -f "$copied_root/$required" ]; then
    exit 14
  fi
done

for required in \
  "launch-dry-run.txt" \
  "doctor.json" \
  "no-llm-response.json" \
  "no-llm-evidence-follow.txt" \
  "working-follow.txt" \
  "policy.json" \
  "adapter-response.json" \
  "adapter-evidence-follow.txt" \
  "launch-fail-closed.txt"
do
  if [ ! -f "$dev_front_copied_root/$required" ]; then
    exit 15
  fi
done

python3 - "$copied_root/home-ok/ledger/evidence/EVID-1/policy.json" "$copied_root/home-ok/memory" "$dev_front_copied_root/doctor.json" <<'PY'
import json
import pathlib
import sys

bringup_policy_path = pathlib.Path(sys.argv[1])
bringup_memory_root = pathlib.Path(sys.argv[2])
dev_front_doctor_path = pathlib.Path(sys.argv[3])

if not bringup_policy_path.is_file():
    raise SystemExit(16)

bringup_policy = json.loads(bringup_policy_path.read_text(encoding="utf-8"))
if bringup_policy.get("binding_id") != "cyrune-free-default":
    raise SystemExit(17)
if bringup_policy.get("memory_state_roots") is not None:
    raise SystemExit(18)
adapters = bringup_policy.get("resolved_kernel_adapters") or {}
expected = {
    "working_store_adapter_id": "memory-kv-inmem",
    "processing_store_adapter_id": "memory-kv-inmem",
    "permanent_store_adapter_id": "memory-kv-inmem",
    "vector_index_adapter_id": "memory-kv-inmem",
    "embedding_engine_ref": "crane-embed-null.v0.1",
}
for key, value in expected.items():
    if adapters.get(key) != value:
        raise SystemExit(19)
if bringup_memory_root.exists():
    raise SystemExit(20)

doctor = json.loads(dev_front_doctor_path.read_text(encoding="utf-8"))
if doctor.get("status") != "healthy":
    raise SystemExit(21)
if doctor.get("bundle_ready") is not False:
    raise SystemExit(22)
if doctor.get("bundle_root") != "":
    raise SystemExit(23)
if doctor.get("distribution_root") != "":
    raise SystemExit(24)
PY
