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

artifact_root="$workspace_root/target/fixed-problem/RC-3/RC3-T3"
distribution_root="$artifact_root/distribution"
bundle_root="$distribution_root/bundle"
home_root="$artifact_root/home"
build_log="$artifact_root/build.txt"
daemon_test_log="$artifact_root/daemon-test.txt"
view_default_file="$artifact_root/view-policy-default.json"
view_alt_file="$artifact_root/view-policy-alt.json"
run_alt_file="$artifact_root/run-alt.json"
run_alt_policy_file="$artifact_root/run-alt-policy.json"
proof_summary_file="$artifact_root/view-policy-routing-proof.txt"

if ! rm -rf "$artifact_root"; then
  exit 12
fi
if ! mkdir -p \
  "$bundle_root/adapter" \
  "$bundle_root/registry/execution-adapters/approved/profiles" \
  "$bundle_root/runtime/ipc" \
  "$home_root"; then
  exit 12
fi

cargo build -p cyrune-daemon -p cyrune-runtime-cli >"$build_log" 2>&1

cp -R "$project_root/Adapter/v0.1/0/catalog" "$bundle_root/adapter/catalog"
mkdir -p "$bundle_root/adapter/policies" "$bundle_root/adapter/bindings"
cp \
  "$project_root/Adapter/v0.1/0/policies/cyrune-free-default.v0.1.json" \
  "$bundle_root/adapter/policies/cyrune-free-default.v0.1.json"
cp \
  "$project_root/Adapter/v0.1/0/bindings/cyrune-free-default.v0.1.json" \
  "$bundle_root/adapter/bindings/cyrune-free-default.v0.1.json"

cat >"$bundle_root/adapter/policies/cyrune-free-alt.v0.1.json" <<'JSON'
{
  "distro_id": "cyrune-free",
  "policy_pack_id": "cyrune-free-alt",
  "version": "v0.1",
  "layers": {
    "working": { "target_items": 10, "ttl_ms": 3600000, "eviction_strategy": "priority" },
    "processing": { "target_items": 20000, "ttl_ms": 3628800000, "promotion_threshold": 0.8 },
    "permanent": { "retention_mode": "immutable" }
  },
  "fail_closed": {
    "on_capacity_out_of_range": true,
    "on_ttl_out_of_range": true,
    "on_missing_capability": true
  }
}
JSON

cat >"$distribution_root/RELEASE_MANIFEST.json" <<'JSON'
{
  "bundle_root_path": "bundle"
}
JSON

cat >"$bundle_root/registry/execution-adapters/approved/registry.json" <<'JSON'
{
  "registry_version": "cyrune.free.execution-adapter-registry.v1",
  "entries": [
    {
      "adapter_id": "local-cli-single-process.v0.1",
      "state": "approved",
      "profile_path": "profiles/local-cli-single-process.v0.1.json"
    }
  ]
}
JSON

cat >"$bundle_root/registry/execution-adapters/approved/profiles/local-cli-single-process.v0.1.json" <<'JSON'
{
  "adapter_id": "local-cli-single-process.v0.1",
  "adapter_version": "0.1.0",
  "execution_kind": "process_stdio",
  "launcher_path": "runtime/ipc/local-cli-single-process.sh",
  "launcher_sha256": "sha256:placeholder",
  "model_id": "model.local",
  "model_revision_or_digest": "sha256:placeholder",
  "allowed_capabilities": ["exec", "fs_read"],
  "default_timeout_s": 120,
  "env_allowlist": []
}
JSON

cat >"$bundle_root/runtime/ipc/local-cli-single-process.sh" <<'SH'
#!/bin/sh
exit 0
SH
chmod +x "$bundle_root/runtime/ipc/local-cli-single-process.sh"

export CYRUNE_HOME="$home_root"
export CYRUNE_DAEMON_BIN="$workspace_root/target/debug/cyrune-daemon"
export CYRUNE_DISTRIBUTION_ROOT="$distribution_root"

"$workspace_root/target/debug/cyrune-runtime-cli" view policy >"$view_default_file"
"$workspace_root/target/debug/cyrune-runtime-cli" view policy --pack cyrune-free-alt >"$view_alt_file"
"$workspace_root/target/debug/cyrune-runtime-cli" run \
  --no-llm \
  --policy-pack cyrune-free-alt \
  --input "rc3 i3 view policy routing proof" \
  >"$run_alt_file"

python3 - "$view_default_file" "$view_alt_file" "$run_alt_file" "$run_alt_policy_file" "$proof_summary_file" "$home_root" <<'PY'
import json
import pathlib
import sys

view_default_path = pathlib.Path(sys.argv[1])
view_alt_path = pathlib.Path(sys.argv[2])
run_alt_path = pathlib.Path(sys.argv[3])
run_alt_policy_path = pathlib.Path(sys.argv[4])
proof_summary_path = pathlib.Path(sys.argv[5])
home_root = pathlib.Path(sys.argv[6])

view_default = json.loads(view_default_path.read_text(encoding="utf-8"))
view_alt = json.loads(view_alt_path.read_text(encoding="utf-8"))
run_alt = json.loads(run_alt_path.read_text(encoding="utf-8"))
evidence_id = run_alt.get("evidence_id")
if not isinstance(evidence_id, str) or evidence_id == "":
    raise SystemExit("run-alt.json must contain non-empty evidence_id")

run_alt_policy_source = (
    home_root / "ledger" / "evidence" / evidence_id / "policy.json"
)
run_alt_policy = json.loads(run_alt_policy_source.read_text(encoding="utf-8"))
run_alt_policy_path.write_text(
    json.dumps(run_alt_policy, indent=2, ensure_ascii=False) + "\n",
    encoding="utf-8",
)

def require(value, expected, label):
    if value != expected:
        raise SystemExit(f"{label} expected {expected!r}, got {value!r}")

require(view_default["requested_policy_pack_id"], "cyrune-free-default", "view default requested")
require(view_default["policy_pack_id"], "cyrune-free-default", "view default resolved")
require(view_default["policy"]["policy_pack_id"], "cyrune-free-default", "view default inner policy")
require(view_alt["requested_policy_pack_id"], "cyrune-free-alt", "view alt requested")
require(view_alt["policy_pack_id"], "cyrune-free-alt", "view alt resolved")
require(view_alt["policy"]["policy_pack_id"], "cyrune-free-alt", "view alt inner policy")
require(run_alt["policy_pack_id"], "cyrune-free-alt", "run alt response policy pack")
require(run_alt_policy["requested_policy_pack_id"], "cyrune-free-alt", "run alt evidence requested")
require(run_alt_policy["policy_pack_id"], "cyrune-free-alt", "run alt evidence resolved")

proof_summary_path.write_text(
    "\n".join(
        [
            "view_default_requested_policy_pack_id=cyrune-free-default",
            "view_default_policy_pack_id=cyrune-free-default",
            "view_default_policy_inner_policy_pack_id=cyrune-free-default",
            "view_alt_requested_policy_pack_id=cyrune-free-alt",
            "view_alt_policy_pack_id=cyrune-free-alt",
            "view_alt_policy_inner_policy_pack_id=cyrune-free-alt",
            "run_alt_response_policy_pack_id=cyrune-free-alt",
            "run_alt_evidence_requested_policy_pack_id=cyrune-free-alt",
            "run_alt_evidence_policy_pack_id=cyrune-free-alt",
        ]
    )
    + "\n",
    encoding="utf-8",
)
PY

cargo test -p cyrune-daemon explain_policy_uses_same_requested_policy_selection_rule_as_run -- --nocapture \
  >"$daemon_test_log" 2>&1

if ! grep -q "explain_policy_uses_same_requested_policy_selection_rule_as_run" "$daemon_test_log"; then
  exit 13
fi
if ! grep -q "test result: ok" "$daemon_test_log"; then
  exit 13
fi
if ! grep -q "view_alt_policy_pack_id=cyrune-free-alt" "$proof_summary_file"; then
  exit 13
fi
