#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PUBLIC_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
FREE_ROOT="$PUBLIC_ROOT"
STATE_ROOT="$FREE_ROOT/target/public-run"
REPORT_PATH="$STATE_ROOT/first-success-report.json"
CYRUNE_HOME="$STATE_ROOT/home"
export CYRUNE_HOME

mkdir -p "$STATE_ROOT"
cd "$FREE_ROOT"
"$STATE_ROOT/bin/cyr" verify first-success | tee "$REPORT_PATH"
