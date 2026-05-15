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

artifact_root="$workspace_root/target/fixed-problem/RC-3/RC3-T2"
proof_file="$artifact_root/retention-predicate-proof.txt"
policy_path="$project_root/Adapter/v0.1/0/policies/cyrune-free-default.v0.1.json"

if ! rm -rf "$artifact_root"; then
  exit 12
fi
if ! mkdir -p "$artifact_root"; then
  exit 12
fi

policy_ttl=$(sed -n '/"processing": {/,/}/s/.*"ttl_ms": \([0-9][0-9]*\),/\1/p' "$policy_path")
if [ -z "$policy_ttl" ]; then
  exit 13
fi

printf 'policy_processing_ttl_ms=%s\n' "$policy_ttl" >"$proof_file"

cargo test \
  --manifest-path "$workspace_root/Cargo.toml" \
  -p cyrune-control-plane \
  memory::tests::shipping_retention_hints_fix_processing_ttl_and_permanent_non_expiring \
  -- --nocapture >>"$proof_file" 2>&1

cargo test \
  --manifest-path "$workspace_root/Cargo.toml" \
  -p cyrune-control-plane \
  memory::tests::shipping_promotion_path_writes_non_expiring_record_to_resolved_permanent_store \
  -- --nocapture >>"$proof_file" 2>&1

for expected in \
  "policy_processing_ttl_ms=3628800000" \
  "processing_ttl_ms=3628800000" \
  "permanent_ttl_ms=null" \
  "permanent_non_expiring=true" \
  "permanent_forgetting_exempt=true" \
  "source_expires_at_present=true" \
  "source_expires_at_unix_ms=20" \
  "promoted_expires_at_present=false" \
  "promoted_non_expiring=true" \
  "promoted_source_evidence_ids=[\"EVID-SHIP-PROM-10\"]" \
  "test memory::tests::shipping_retention_hints_fix_processing_ttl_and_permanent_non_expiring ... ok" \
  "test memory::tests::shipping_promotion_path_writes_non_expiring_record_to_resolved_permanent_store ... ok"
do
  if ! grep -F "$expected" "$proof_file" >/dev/null 2>&1; then
    exit 14
  fi
done

