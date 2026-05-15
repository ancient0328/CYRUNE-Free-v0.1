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

artifact_root="$workspace_root/target/fixed-problem/RC-3/RC3-T4"
base_distribution_root="$artifact_root/base/distribution"
base_bundle_root="$base_distribution_root/bundle"
base_home_root="$artifact_root/base/home"
missing_registry_distribution_root="$artifact_root/missing-registry/distribution"
missing_registry_home_root="$artifact_root/missing-registry/home"
build_log="$artifact_root/build.txt"
daemon_tests_log="$artifact_root/daemon-tests.txt"
canonical_json="$artifact_root/shipping-binding-canonical.json"
shorthand_json="$artifact_root/shipping-binding-shorthand.json"
canonical_policy_json="$artifact_root/shipping-binding-canonical-policy.json"
shorthand_policy_json="$artifact_root/shipping-binding-shorthand-policy.json"
missing_registry_json="$artifact_root/packaged-missing-registry.json"
view_missing_pack_stdout="$artifact_root/view-missing-pack.stdout"
view_missing_pack_stderr="$artifact_root/view-missing-pack.stderr"
view_missing_pack_status="$artifact_root/view-missing-pack.status"
explain_policy_ipc_json="$artifact_root/explain-policy-missing-pack-ipc.json"
proof_summary_file="$artifact_root/reject-surface-proof.txt"

if ! rm -rf "$artifact_root"; then
  exit 12
fi
if ! mkdir -p \
  "$base_bundle_root/adapter" \
  "$base_bundle_root/registry/execution-adapters/approved/profiles" \
  "$base_bundle_root/runtime/ipc" \
  "$base_home_root" \
  "$missing_registry_home_root"; then
  exit 12
fi

cargo build -p cyrune-daemon -p cyrune-runtime-cli >"$build_log" 2>&1

cp -R "$project_root/Adapter/v0.1/0/catalog" "$base_bundle_root/adapter/catalog"
mkdir -p "$base_bundle_root/adapter/policies" "$base_bundle_root/adapter/bindings"
cp \
  "$project_root/Adapter/v0.1/0/policies/cyrune-free-default.v0.1.json" \
  "$base_bundle_root/adapter/policies/cyrune-free-default.v0.1.json"
cp \
  "$project_root/Adapter/v0.1/0/bindings/cyrune-free-default.v0.1.json" \
  "$base_bundle_root/adapter/bindings/cyrune-free-default.v0.1.json"

cat >"$base_distribution_root/RELEASE_MANIFEST.json" <<'JSON'
{
  "bundle_root_path": "bundle"
}
JSON

cat >"$base_bundle_root/registry/execution-adapters/approved/registry.json" <<'JSON'
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

cat >"$base_bundle_root/registry/execution-adapters/approved/profiles/local-cli-single-process.v0.1.json" <<'JSON'
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

cat >"$base_bundle_root/runtime/ipc/local-cli-single-process.sh" <<'SH'
#!/bin/sh
exit 0
SH
chmod +x "$base_bundle_root/runtime/ipc/local-cli-single-process.sh"

mkdir -p "$(dirname "$missing_registry_distribution_root")"
cp -R "$base_distribution_root" "$missing_registry_distribution_root"
rm "$missing_registry_distribution_root/bundle/registry/execution-adapters/approved/registry.json"

export CYRUNE_DAEMON_BIN="$workspace_root/target/debug/cyrune-daemon"

export CYRUNE_HOME="$base_home_root"
export CYRUNE_DISTRIBUTION_ROOT="$base_distribution_root"

"$workspace_root/target/debug/cyrune-runtime-cli" run \
  --no-llm \
  --binding cyrune-free-shipping.v0.1 \
  --input "rc3 i4 canonical binding proof" \
  >"$canonical_json"

"$workspace_root/target/debug/cyrune-runtime-cli" run \
  --no-llm \
  --binding cyrune-free-shipping \
  --input "rc3 i4 shorthand binding proof" \
  >"$shorthand_json"

set +e
"$workspace_root/target/debug/cyrune-runtime-cli" view policy --pack missing-pack \
  >"$view_missing_pack_stdout" \
  2>"$view_missing_pack_stderr"
view_missing_pack_exit=$?
set -e
printf '%s\n' "$view_missing_pack_exit" >"$view_missing_pack_status"

printf '%s\n' \
  '{"version":"cyrune.free.ipc.v1","message_id":"MSG-RC3-T4-1","command":"ExplainPolicy","payload":{"policy_pack":"missing-pack","last_denial_id":null}}' \
  | "$workspace_root/target/debug/cyrune-daemon" serve-stdio \
  >"$explain_policy_ipc_json"

export CYRUNE_HOME="$missing_registry_home_root"
export CYRUNE_DISTRIBUTION_ROOT="$missing_registry_distribution_root"

"$workspace_root/target/debug/cyrune-runtime-cli" run \
  --adapter local-cli-single-process.v0.1 \
  --input "rc3 i4 packaged missing registry proof" \
  >"$missing_registry_json"

python3 - \
  "$canonical_json" \
  "$shorthand_json" \
  "$canonical_policy_json" \
  "$shorthand_policy_json" \
  "$missing_registry_json" \
  "$view_missing_pack_stdout" \
  "$view_missing_pack_stderr" \
  "$view_missing_pack_status" \
  "$explain_policy_ipc_json" \
  "$proof_summary_file" \
  "$base_home_root" \
  "$artifact_root" <<'PY'
import json
import pathlib
import sys

canonical_json = pathlib.Path(sys.argv[1])
shorthand_json = pathlib.Path(sys.argv[2])
canonical_policy_json = pathlib.Path(sys.argv[3])
shorthand_policy_json = pathlib.Path(sys.argv[4])
missing_registry_json = pathlib.Path(sys.argv[5])
view_missing_pack_stdout = pathlib.Path(sys.argv[6])
view_missing_pack_stderr = pathlib.Path(sys.argv[7])
view_missing_pack_status = pathlib.Path(sys.argv[8])
explain_policy_ipc_json = pathlib.Path(sys.argv[9])
proof_summary_file = pathlib.Path(sys.argv[10])
base_home_root = pathlib.Path(sys.argv[11])
artifact_root = pathlib.Path(sys.argv[12])

canonical = json.loads(canonical_json.read_text(encoding="utf-8"))
shorthand = json.loads(shorthand_json.read_text(encoding="utf-8"))
missing_registry = json.loads(missing_registry_json.read_text(encoding="utf-8"))
view_status = view_missing_pack_status.read_text(encoding="utf-8").strip()
view_stderr = view_missing_pack_stderr.read_text(encoding="utf-8").strip()
view_stdout = view_missing_pack_stdout.read_text(encoding="utf-8")
ipc_response = json.loads(explain_policy_ipc_json.read_text(encoding="utf-8"))

for label, payload in [("canonical", canonical), ("shorthand", shorthand)]:
    if payload.get("reason_kind") != "binding_unresolved":
        raise SystemExit(f"{label} reason_kind must be binding_unresolved")
    if payload.get("rule_id") != "BND-003":
        raise SystemExit(f"{label} rule_id must be BND-003")

if canonical.get("message") != shorthand.get("message"):
    raise SystemExit("canonical/shorthand messages must match")
if canonical.get("remediation") != shorthand.get("remediation"):
    raise SystemExit("canonical/shorthand remediations must match")

canonical_policy_source = (
    base_home_root / "ledger" / "evidence" / canonical["evidence_id"] / "policy.json"
)
shorthand_policy_source = (
    base_home_root / "ledger" / "evidence" / shorthand["evidence_id"] / "policy.json"
)
canonical_policy = json.loads(canonical_policy_source.read_text(encoding="utf-8"))
shorthand_policy = json.loads(shorthand_policy_source.read_text(encoding="utf-8"))
canonical_policy_json.write_text(
    json.dumps(canonical_policy, indent=2, ensure_ascii=False) + "\n",
    encoding="utf-8",
)
shorthand_policy_json.write_text(
    json.dumps(shorthand_policy, indent=2, ensure_ascii=False) + "\n",
    encoding="utf-8",
)
if canonical_policy.get("binding_id") != "cyrune-free-shipping.v0.1":
    raise SystemExit("canonical policy binding_id must be canonical shipping id")
if shorthand_policy.get("binding_id") != "cyrune-free-shipping.v0.1":
    raise SystemExit("shorthand policy binding_id must be canonical shipping id")

if missing_registry.get("reason_kind") != "binding_unresolved":
    raise SystemExit("missing registry reason_kind must be binding_unresolved")
if missing_registry.get("rule_id") != "BND-004":
    raise SystemExit("missing registry rule_id must be BND-004")

for field_name in ["message", "remediation"]:
    field_value = missing_registry.get(field_name, "")
    if not isinstance(field_value, str):
        raise SystemExit(f"missing registry {field_name} must be a string")
    for forbidden in [str(artifact_root), str(base_home_root)]:
        if forbidden and forbidden in field_value:
            raise SystemExit(f"missing registry {field_name} leaked path {forbidden}")

if view_status != "1":
    raise SystemExit("view missing-pack exit status must be 1")
if view_stdout != "":
    raise SystemExit("view missing-pack stdout must stay empty")
if view_stderr != "requested policy pack is unresolved: missing-pack":
    raise SystemExit("view missing-pack stderr must be sanitized public error")
if "policy exact match not found" in view_stderr:
    raise SystemExit("view missing-pack stderr leaked internal resolver detail")
for forbidden in [str(artifact_root), str(base_home_root)]:
    if forbidden and forbidden in view_stderr:
        raise SystemExit(f"view missing-pack stderr leaked path {forbidden}")

if ipc_response.get("status") != "error":
    raise SystemExit("ExplainPolicy IPC response must be error")
ipc_message = ipc_response.get("payload", {}).get("message")
if ipc_message != "requested policy pack is unresolved: missing-pack":
    raise SystemExit("ExplainPolicy IPC message must be sanitized public error")
if "policy exact match not found" in ipc_message:
    raise SystemExit("ExplainPolicy IPC message leaked internal resolver detail")

proof_summary_file.write_text(
    "\n".join(
        [
            "canonical_reason_kind=binding_unresolved",
            "canonical_rule_id=BND-003",
            "canonical_policy_binding_id=cyrune-free-shipping.v0.1",
            "shorthand_reason_kind=binding_unresolved",
            "shorthand_rule_id=BND-003",
            "shorthand_policy_binding_id=cyrune-free-shipping.v0.1",
            "normalized_public_surface_equal=true",
            "packaged_missing_registry_reason_kind=binding_unresolved",
            "packaged_missing_registry_rule_id=BND-004",
            "packaged_missing_registry_no_path_leak=true",
            "view_missing_pack_exit_status=1",
            "view_missing_pack_public_message=requested policy pack is unresolved: missing-pack",
            "view_missing_pack_internal_detail_leak=false",
            "explain_policy_ipc_status=error",
            "explain_policy_ipc_public_message=requested policy pack is unresolved: missing-pack",
            "explain_policy_ipc_internal_detail_leak=false",
        ]
    )
    + "\n",
    encoding="utf-8",
)
PY

{
  cargo test -p cyrune-daemon run_normalizes_missing_shipping_binding_rejection_for_shorthand_and_canonical_requests -- --nocapture
  cargo test -p cyrune-daemon run_rejects_missing_packaged_registry_without_path_leakage -- --nocapture
  cargo test -p cyrune-daemon serve_stdio_returns_error_response_for_unresolved_explain_policy -- --nocapture
} >"$daemon_tests_log" 2>&1

for pattern in \
  "run_normalizes_missing_shipping_binding_rejection_for_shorthand_and_canonical_requests ... ok" \
  "run_rejects_missing_packaged_registry_without_path_leakage ... ok" \
  "serve_stdio_returns_error_response_for_unresolved_explain_policy ... ok" \
  "test result: ok"; do
  if ! grep -q "$pattern" "$daemon_tests_log"; then
    exit 13
  fi
done

if ! grep -q "packaged_missing_registry_no_path_leak=true" "$proof_summary_file"; then
  exit 13
fi
