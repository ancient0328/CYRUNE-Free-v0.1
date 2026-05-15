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

artifact_root="$workspace_root/target/shipping-memory/SM-C/SMC-T3"
proof_file="$artifact_root/promotion-non-expiring-proof.txt"

if ! rm -rf "$artifact_root"; then
  exit 12
fi
if ! mkdir -p "$artifact_root"; then
  exit 12
fi

cargo test \
  --manifest-path "$workspace_root/Cargo.toml" \
  -p cyrune-control-plane \
  memory::tests::shipping_promotion_path_writes_non_expiring_record_to_resolved_permanent_store \
  -- --nocapture >"$proof_file" 2>&1

for expected in \
  "correlation_id=RUN-20260406-0001" \
  "binding_id=cyrune-free-shipping.v0.1" \
  "permanent_adapter=memory-stoolap-permanent" \
  "promotion_source_record_id=MEM-SHIP-PROMOTE-001" \
  "source_expires_at_present=true" \
  "promoted_record_id=MEM-SHIP-KNOW-001" \
  "promoted_source_evidence_ids=[\"EVID-SHIP-PROM-10\"]" \
  "promoted_non_expiring=true" \
  "test memory::tests::shipping_promotion_path_writes_non_expiring_record_to_resolved_permanent_store ... ok"
do
  if ! grep -F "$expected" "$proof_file" >/dev/null 2>&1; then
    exit 13
  fi
done

if ! grep -E '^permanent_state_root=/.*\/permanent$' "$proof_file" >/dev/null 2>&1; then
  exit 14
fi
