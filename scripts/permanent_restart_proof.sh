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

artifact_root="$workspace_root/target/shipping-memory/SM-C/SMC-T2"
proof_file="$artifact_root/permanent-restart-proof.txt"

if ! rm -rf "$artifact_root"; then
  exit 12
fi
if ! mkdir -p "$artifact_root"; then
  exit 12
fi

cargo test \
  --manifest-path "$workspace_root/Cargo.toml" \
  -p cyrune-control-plane \
  memory::tests::shipping_permanent_records_materialize_to_stoolap_root \
  -- --nocapture >"$proof_file" 2>&1

for expected in \
  "correlation_id=RUN-20260406-0001" \
  "binding_id=cyrune-free-shipping.v0.1" \
  "permanent_adapter=memory-stoolap-permanent" \
  "reopened_record_id=MEM-SHIP-PERM-001" \
  "reopened_payload_ref=permanent://knowledge_records/MEM-SHIP-PERM-001" \
  "validity_state=\"invalidated\"" \
  "relation_count=1" \
  "non_expiring=true" \
  "test memory::tests::shipping_permanent_records_materialize_to_stoolap_root ... ok"
do
  if ! grep -F "$expected" "$proof_file" >/dev/null 2>&1; then
    exit 13
  fi
done

if ! grep -E '^permanent_state_root=/.*\/permanent$' "$proof_file" >/dev/null 2>&1; then
  exit 14
fi
