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
if ! project_root=$(CDPATH= cd -- "$workspace_root/../../../../.." 2>/dev/null && pwd); then
  exit 11
fi

stage_script="$script_dir/stage_shipping_readiness.py"
shipping_root="$workspace_root/target/shipping/S2/cyrune-free-v0.1"
artifact_root="$workspace_root/target/fixed-problem/RC-3/RC3-T1"
stage_log="$artifact_root/stage.log"
positive_distribution_root="$artifact_root/positive/distribution"
positive_home_root="$artifact_root/positive/home"
positive_warmup_json="$artifact_root/positive/accepted-warmup.json"
positive_followup_json="$artifact_root/positive/accepted-followup.json"
positive_policy_json="$artifact_root/positive/accepted-policy.json"
positive_bundle_manifest_json="$artifact_root/positive/bundle-manifest.json"
positive_materialized_manifest_json="$artifact_root/positive/materialized-manifest.json"
positive_summary_file="$artifact_root/positive/exact-pin-positive.txt"
negative_source_distribution_root="$artifact_root/negative-source/distribution"
negative_source_home_root="$artifact_root/negative-source/home"
negative_source_json="$artifact_root/negative-source/rejected.json"
negative_source_policy_json="$artifact_root/negative-source/rejected-policy.json"
negative_source_summary_file="$artifact_root/negative-source/exact-pin-negative-source.txt"
negative_runtime_json="$artifact_root/negative-runtime/rejected.json"
negative_runtime_policy_json="$artifact_root/negative-runtime/rejected-policy.json"
negative_runtime_manifest_json="$artifact_root/negative-runtime/materialized-manifest-invalid.json"
negative_runtime_summary_file="$artifact_root/negative-runtime/exact-pin-negative-runtime.txt"
control_plane_tests_log="$artifact_root/control-plane-tests.txt"
daemon_tests_log="$artifact_root/daemon-tests.txt"

if ! command -v python3 >/dev/null 2>&1; then
  exit 16
fi

if ! rm -rf "$artifact_root"; then
  exit 12
fi
if ! mkdir -p \
  "$artifact_root/positive" \
  "$artifact_root/negative-source" \
  "$artifact_root/negative-runtime"; then
  exit 12
fi

python3 "$stage_script" >"$stage_log" 2>&1

if [ ! -d "$shipping_root" ]; then
  exit 12
fi

cp -R "$shipping_root" "$positive_distribution_root"

positive_cyr="$positive_distribution_root/bin/cyr"
if [ ! -x "$positive_cyr" ]; then
  exit 12
fi

export CRANE_ROOT="/nonexistent/should-not-be-used"

CYRUNE_DISTRIBUTION_ROOT="$positive_distribution_root" \
CYRUNE_HOME="$positive_home_root" \
  "$positive_cyr" run \
  --no-llm \
  --binding cyrune-free-shipping.v0.1 \
  --input "rc3 t1 exact pin positive warmup" \
  >"$positive_warmup_json"

CYRUNE_DISTRIBUTION_ROOT="$positive_distribution_root" \
CYRUNE_HOME="$positive_home_root" \
  "$positive_cyr" run \
  --no-llm \
  --binding cyrune-free-shipping.v0.1 \
  --input "rc3 t1 exact pin positive followup" \
  >"$positive_followup_json"

python3 - \
  "$positive_followup_json" \
  "$positive_policy_json" \
  "$positive_bundle_manifest_json" \
  "$positive_materialized_manifest_json" \
  "$positive_summary_file" \
  "$positive_distribution_root" \
  "$positive_home_root" <<'PY'
import json
import pathlib
import sys


def require_string(payload, key):
    value = payload.get(key)
    if not isinstance(value, str) or value == "":
        raise SystemExit(f"missing non-empty string field: {key}")
    return value


def engine_ref_from_manifest(manifest):
    model_id = "".join(
        char if char.isalnum() else "-"
        for char in manifest["upstream_model_id"]
    )
    revision = "".join(
        char if char.isalnum() else "-"
        for char in manifest["upstream_revision"]
    )
    return f"embedding-{manifest['engine_kind']}-{model_id}-{revision}.v0.1"


followup_path = pathlib.Path(sys.argv[1])
policy_copy_path = pathlib.Path(sys.argv[2])
bundle_manifest_copy_path = pathlib.Path(sys.argv[3])
materialized_manifest_copy_path = pathlib.Path(sys.argv[4])
summary_path = pathlib.Path(sys.argv[5])
distribution_root = pathlib.Path(sys.argv[6])
home_root = pathlib.Path(sys.argv[7])

followup = json.loads(followup_path.read_text(encoding="utf-8"))
evidence_id = require_string(followup, "evidence_id")
require_string(followup, "response_to")
require_string(followup, "correlation_id")
require_string(followup, "run_id")
require_string(followup, "policy_pack_id")
if followup["policy_pack_id"] != "cyrune-free-default":
    raise SystemExit("positive followup must keep default policy pack")

policy_source_path = home_root / "ledger" / "evidence" / evidence_id / "policy.json"
bundle_manifest_path = (
    distribution_root
    / "share"
    / "cyrune"
    / "bundle-root"
    / "embedding"
    / "exact-pins"
    / "cyrune-free-shipping.v0.1.json"
)
materialized_manifest_path = (
    home_root
    / "embedding"
    / "exact-pins"
    / "cyrune-free-shipping.v0.1.json"
)
processing_db = home_root / "memory" / "processing" / "processing.redb"
permanent_root = home_root / "memory" / "permanent"

policy = json.loads(policy_source_path.read_text(encoding="utf-8"))
bundle_manifest = json.loads(bundle_manifest_path.read_text(encoding="utf-8"))
materialized_manifest = json.loads(materialized_manifest_path.read_text(encoding="utf-8"))
expected_engine_ref = engine_ref_from_manifest(bundle_manifest)

if policy.get("binding_id") != "cyrune-free-shipping.v0.1":
    raise SystemExit("positive policy binding_id must stay shipping canonical id")
resolved_adapters = policy.get("resolved_kernel_adapters", {})
if resolved_adapters.get("embedding_engine_ref") != expected_engine_ref:
    raise SystemExit("positive policy embedding_engine_ref must match source-driven engine ref")
if resolved_adapters.get("processing_store_adapter_id") != "memory-redb-processing":
    raise SystemExit("positive policy processing adapter mismatch")
if resolved_adapters.get("permanent_store_adapter_id") != "memory-stoolap-permanent":
    raise SystemExit("positive policy permanent adapter mismatch")

exact_pin = policy.get("embedding_exact_pin")
if not isinstance(exact_pin, dict):
    raise SystemExit("positive policy embedding_exact_pin must be present")
for key in [
    "engine_kind",
    "upstream_model_id",
    "artifact_set",
    "artifact_sha256",
    "dimensions",
    "pooling",
    "normalization",
    "prompt_profile",
    "token_limit",
    "distance",
]:
    if exact_pin.get(key) != bundle_manifest.get(key):
        raise SystemExit(f"positive policy exact pin mismatch: {key}")
if exact_pin.get("upstream_revision") != bundle_manifest.get("upstream_revision"):
    raise SystemExit("positive policy upstream_revision mismatch")

memory_state_roots = policy.get("memory_state_roots")
if not isinstance(memory_state_roots, dict):
    raise SystemExit("positive policy memory_state_roots must be present")
if memory_state_roots.get("processing_state_root") != str(home_root / "memory" / "processing"):
    raise SystemExit("positive processing_state_root mismatch")
if memory_state_roots.get("permanent_state_root") != str(home_root / "memory" / "permanent"):
    raise SystemExit("positive permanent_state_root mismatch")

if bundle_manifest != materialized_manifest:
    raise SystemExit("materialized manifest must match bundle manifest byte-for-byte at JSON level")
if not processing_db.is_file():
    raise SystemExit("processing redb must exist after positive exact-pin run")
if not permanent_root.is_dir():
    raise SystemExit("permanent root must exist after positive exact-pin run")

policy_copy_path.write_text(
    json.dumps(policy, ensure_ascii=False, indent=2) + "\n",
    encoding="utf-8",
)
bundle_manifest_copy_path.write_text(
    json.dumps(bundle_manifest, ensure_ascii=False, indent=2) + "\n",
    encoding="utf-8",
)
materialized_manifest_copy_path.write_text(
    json.dumps(materialized_manifest, ensure_ascii=False, indent=2) + "\n",
    encoding="utf-8",
)
summary_path.write_text(
    "\n".join(
        [
            f"evidence_id={evidence_id}",
            f"embedding_engine_ref={expected_engine_ref}",
            f"upstream_revision={bundle_manifest['upstream_revision']}",
            f"processing_db={processing_db}",
            f"permanent_root={permanent_root}",
            f"materialized_manifest={materialized_manifest_path}",
        ]
    )
    + "\n",
    encoding="utf-8",
)
PY

cp -R "$shipping_root" "$negative_source_distribution_root"

python3 - "$negative_source_distribution_root" <<'PY'
import json
import pathlib
import sys

distribution_root = pathlib.Path(sys.argv[1])
manifest_path = (
    distribution_root
    / "share"
    / "cyrune"
    / "bundle-root"
    / "embedding"
    / "exact-pins"
    / "cyrune-free-shipping.v0.1.json"
)
manifest = json.loads(manifest_path.read_text(encoding="utf-8"))
manifest["artifact_sha256"]["model.onnx"] = "0" * 64
manifest_path.write_text(
    json.dumps(manifest, ensure_ascii=False, indent=2) + "\n",
    encoding="utf-8",
)
PY

negative_source_cyr="$negative_source_distribution_root/bin/cyr"
if [ ! -x "$negative_source_cyr" ]; then
  exit 12
fi

CYRUNE_DISTRIBUTION_ROOT="$negative_source_distribution_root" \
CYRUNE_HOME="$negative_source_home_root" \
  "$negative_source_cyr" run \
  --no-llm \
  --binding cyrune-free-shipping.v0.1 \
  --input "rc3 t1 exact pin negative source" \
  >"$negative_source_json"

python3 - \
  "$negative_source_json" \
  "$negative_source_policy_json" \
  "$negative_source_summary_file" \
  "$negative_source_home_root" \
  "$artifact_root" <<'PY'
import json
import pathlib
import sys

reject_path = pathlib.Path(sys.argv[1])
policy_copy_path = pathlib.Path(sys.argv[2])
summary_path = pathlib.Path(sys.argv[3])
home_root = pathlib.Path(sys.argv[4])
artifact_root = pathlib.Path(sys.argv[5])

reject = json.loads(reject_path.read_text(encoding="utf-8"))
if reject.get("reason_kind") != "binding_unresolved":
    raise SystemExit("source-negative reason_kind must be binding_unresolved")
if reject.get("rule_id") != "BND-006":
    raise SystemExit("source-negative rule_id must be BND-006")
message = reject.get("message", "")
if "shipping exact pin authoritative source is invalid: artifact hash mismatch: model.onnx" not in message:
    raise SystemExit("source-negative message must mention model.onnx hash mismatch")
for forbidden in [str(artifact_root), str(home_root)]:
    if forbidden and forbidden in message:
        raise SystemExit(f"source-negative message leaked path {forbidden}")

evidence_id = reject.get("evidence_id")
policy_source_path = home_root / "ledger" / "evidence" / evidence_id / "policy.json"
policy = json.loads(policy_source_path.read_text(encoding="utf-8"))
if policy.get("binding_id") != "cyrune-free-shipping.v0.1":
    raise SystemExit("source-negative policy binding_id mismatch")
if policy["resolved_kernel_adapters"]["processing_store_adapter_id"] != "unresolved":
    raise SystemExit("source-negative processing adapter must stay unresolved")
if policy["resolved_kernel_adapters"]["permanent_store_adapter_id"] != "unresolved":
    raise SystemExit("source-negative permanent adapter must stay unresolved")
if policy.get("embedding_exact_pin") is not None:
    raise SystemExit("source-negative embedding_exact_pin must stay null")
if policy.get("memory_state_roots") is not None:
    raise SystemExit("source-negative memory_state_roots must stay null")

policy_copy_path.write_text(
    json.dumps(policy, ensure_ascii=False, indent=2) + "\n",
    encoding="utf-8",
)
summary_path.write_text(
    "\n".join(
        [
            f"evidence_id={evidence_id}",
            f"rule_id={reject['rule_id']}",
            "negative_kind=resolver_source_invalid",
        ]
    )
    + "\n",
    encoding="utf-8",
)
PY

python3 - "$positive_home_root" "$negative_runtime_manifest_json" <<'PY'
import json
import pathlib
import sys

home_root = pathlib.Path(sys.argv[1])
invalid_manifest_copy_path = pathlib.Path(sys.argv[2])
manifest_path = (
    home_root
    / "embedding"
    / "exact-pins"
    / "cyrune-free-shipping.v0.1.json"
)
manifest = json.loads(manifest_path.read_text(encoding="utf-8"))
manifest["artifact_sha256"]["model.onnx"] = "0" * 64
manifest_path.write_text(
    json.dumps(manifest, ensure_ascii=False, indent=2) + "\n",
    encoding="utf-8",
)
invalid_manifest_copy_path.write_text(
    json.dumps(manifest, ensure_ascii=False, indent=2) + "\n",
    encoding="utf-8",
)
PY

CYRUNE_DISTRIBUTION_ROOT="$positive_distribution_root" \
CYRUNE_HOME="$positive_home_root" \
  "$positive_cyr" run \
  --no-llm \
  --binding cyrune-free-shipping.v0.1 \
  --input "rc3 t1 exact pin negative runtime" \
  >"$negative_runtime_json"

python3 - \
  "$negative_runtime_json" \
  "$negative_runtime_policy_json" \
  "$negative_runtime_summary_file" \
  "$positive_home_root" \
  "$artifact_root" <<'PY'
import json
import pathlib
import sys

reject_path = pathlib.Path(sys.argv[1])
policy_copy_path = pathlib.Path(sys.argv[2])
summary_path = pathlib.Path(sys.argv[3])
home_root = pathlib.Path(sys.argv[4])
artifact_root = pathlib.Path(sys.argv[5])

reject = json.loads(reject_path.read_text(encoding="utf-8"))
if reject.get("reason_kind") != "binding_unresolved":
    raise SystemExit("runtime-negative reason_kind must be binding_unresolved")
if reject.get("rule_id") != "BND-010":
    raise SystemExit("runtime-negative rule_id must be BND-010")
message = reject.get("message")
if message != "shipping memory retrieval source is unresolved":
    raise SystemExit("runtime-negative message must stay sanitized BND-010 message")
remediation = reject.get("remediation")
if remediation != "shipping memory backend / retrieval source を修正して再実行する":
    raise SystemExit("runtime-negative remediation mismatch")
for field_value in [message, remediation]:
    for forbidden in [str(artifact_root), str(home_root)]:
        if forbidden and forbidden in field_value:
            raise SystemExit(f"runtime-negative public field leaked path {forbidden}")

evidence_id = reject.get("evidence_id")
policy_source_path = home_root / "ledger" / "evidence" / evidence_id / "policy.json"
policy = json.loads(policy_source_path.read_text(encoding="utf-8"))
if policy.get("binding_id") != "cyrune-free-shipping.v0.1":
    raise SystemExit("runtime-negative policy binding_id mismatch")
if policy["resolved_kernel_adapters"]["processing_store_adapter_id"] != "unresolved":
    raise SystemExit("runtime-negative processing adapter must stay unresolved")
if policy["resolved_kernel_adapters"]["permanent_store_adapter_id"] != "unresolved":
    raise SystemExit("runtime-negative permanent adapter must stay unresolved")
if policy.get("embedding_exact_pin") is not None:
    raise SystemExit("runtime-negative embedding_exact_pin must stay null")
if policy.get("memory_state_roots") is not None:
    raise SystemExit("runtime-negative memory_state_roots must stay null")

policy_copy_path.write_text(
    json.dumps(policy, ensure_ascii=False, indent=2) + "\n",
    encoding="utf-8",
)
summary_path.write_text(
    "\n".join(
        [
            f"evidence_id={evidence_id}",
            f"rule_id={reject['rule_id']}",
            "negative_kind=materialized_runtime_invalid",
        ]
    )
    + "\n",
    encoding="utf-8",
)
PY

cargo test -p cyrune-control-plane shipping_binding_resolves_source_driven_non_null_engine_ref -- --nocapture \
  >"$control_plane_tests_log" 2>&1
cargo test -p cyrune-control-plane shipping_selection_uses_resolved_processing_and_permanent_stores -- --nocapture \
  >>"$control_plane_tests_log" 2>&1

export CRANE_ROOT="$project_root"

cargo test -p cyrune-daemon run_rejects_shipping_binding_when_exact_pin_source_is_missing -- --nocapture \
  >"$daemon_tests_log" 2>&1
cargo test -p cyrune-daemon run_prefers_exact_pin_source_unresolved_before_memory_backend_checks -- --nocapture \
  >>"$daemon_tests_log" 2>&1

if ! grep -q "shipping_binding_resolves_source_driven_non_null_engine_ref" "$control_plane_tests_log"; then
  exit 13
fi
if ! grep -q "shipping_selection_uses_resolved_processing_and_permanent_stores" "$control_plane_tests_log"; then
  exit 13
fi
if ! grep -q "test result: ok" "$control_plane_tests_log"; then
  exit 13
fi
if ! grep -q "run_rejects_shipping_binding_when_exact_pin_source_is_missing" "$daemon_tests_log"; then
  exit 13
fi
if ! grep -q "run_prefers_exact_pin_source_unresolved_before_memory_backend_checks" "$daemon_tests_log"; then
  exit 13
fi
if ! grep -q "test result: ok" "$daemon_tests_log"; then
  exit 13
fi
