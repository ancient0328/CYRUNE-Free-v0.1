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

bootstrap_script="$script_dir/dev_front_bootstrap.sh"
launch_script="$script_dir/dev_front_launch.sh"

set +e
"$bootstrap_script" >/dev/null 2>/dev/null
status=$?
set -e
if [ "$status" -ne 0 ]; then
  exit "$status"
fi

dev_front_root="$workspace_root/target/developer-demo-front"
env_file="$dev_front_root/env.sh"
if [ ! -f "$env_file" ]; then
  exit 14
fi

if ! . "$env_file"; then
  exit 14
fi

if ! command -v python3 >/dev/null 2>&1; then
  exit 16
fi

artifact_root="$CYRUNE_DEV_FRONT_ROOT/proof/D3"
workspace_dir="$CYRUNE_DEV_FRONT_ROOT/workspace"
cyr_bin="$CYRUNE_DEV_FRONT_ROOT/bin/cyr"
config_file="$CYRUNE_HOME/terminal/config/wezterm.lua"

if ! rm -rf "$artifact_root"; then
  exit 12
fi
if ! mkdir -p "$artifact_root" "$workspace_dir"; then
  exit 12
fi

launch_dry_run_file="$artifact_root/launch-dry-run.txt"
doctor_file="$artifact_root/doctor.json"
no_llm_response_file="$artifact_root/no-llm-response.json"
no_llm_evidence_file="$artifact_root/no-llm-evidence-follow.txt"
working_follow_file="$artifact_root/working-follow.txt"
policy_file="$artifact_root/policy.json"
adapter_response_file="$artifact_root/adapter-response.json"
adapter_evidence_file="$artifact_root/adapter-evidence-follow.txt"
launch_fail_closed_file="$artifact_root/launch-fail-closed.txt"

rm -f "$config_file"
"$launch_script" --dry-run >"$launch_dry_run_file"
"$cyr_bin" doctor >"$doctor_file"
"$cyr_bin" run --no-llm --input "developer demo front no-llm proof" >"$no_llm_response_file"

no_llm_correlation_id="$(
  python3 - "$no_llm_response_file" <<'PY'
import json
import sys

with open(sys.argv[1], encoding="utf-8") as handle:
    payload = json.load(handle)

value = payload.get("correlation_id")
if not isinstance(value, str) or value == "":
    raise SystemExit(1)

print(value)
PY
)"

"$cyr_bin" view evidence --follow "$no_llm_correlation_id" >"$no_llm_evidence_file"
"$cyr_bin" view working --follow --interval-ms 100 --max-updates 1 >"$working_follow_file"
"$cyr_bin" view policy >"$policy_file"
"$cyr_bin" run --adapter local-cli-single-process.v0.1 --input "developer demo front approved adapter proof" --cap exec --cap fs_read --cwd "$workspace_dir" >"$adapter_response_file"

adapter_correlation_id="$(
  python3 - "$adapter_response_file" <<'PY'
import json
import sys

with open(sys.argv[1], encoding="utf-8") as handle:
    payload = json.load(handle)

value = payload.get("correlation_id")
if not isinstance(value, str) or value == "":
    raise SystemExit(1)

print(value)
PY
)"

"$cyr_bin" view evidence --follow "$adapter_correlation_id" >"$adapter_evidence_file"

set +e
CYRUNE_DEV_FRONT_WEZTERM_BIN="$CYRUNE_DEV_FRONT_ROOT/bin/wezterm-missing" \
  "$launch_script" --dry-run >/dev/null 2>/dev/null
status=$?
set -e
printf '%s\n' "$status" >"$launch_fail_closed_file"

python3 - \
  "$artifact_root" \
  "$CYRUNE_DEV_FRONT_ROOT" \
  "$CYRUNE_HOME" \
  "$workspace_dir" <<'PY'
import json
import pathlib
import sys

artifact_root = pathlib.Path(sys.argv[1])
dev_front_root = pathlib.Path(sys.argv[2])
home_root = pathlib.Path(sys.argv[3])
workspace_dir = pathlib.Path(sys.argv[4])

required_files = [
    "launch-dry-run.txt",
    "doctor.json",
    "no-llm-response.json",
    "no-llm-evidence-follow.txt",
    "working-follow.txt",
    "policy.json",
    "adapter-response.json",
    "adapter-evidence-follow.txt",
    "launch-fail-closed.txt",
]
for name in required_files:
    path = artifact_root / name
    if not path.is_file():
        raise SystemExit(f"missing artifact: {name}")

launch_dry_run = (artifact_root / "launch-dry-run.txt").read_text(encoding="utf-8")
launch_lines = launch_dry_run.splitlines()
if len(launch_lines) != 1:
    raise SystemExit("launch-dry-run must contain exactly one line")
launch_line = launch_lines[0]
if " start --config-file " not in launch_line:
    raise SystemExit("launch-dry-run missing start --config-file")
expected_config = str(home_root / "terminal" / "config" / "wezterm.lua")
if expected_config not in launch_line:
    raise SystemExit("launch-dry-run missing terminal config path")
if not pathlib.Path(expected_config).is_file():
    raise SystemExit("launch-dry-run must materialize terminal config path")

doctor = json.loads((artifact_root / "doctor.json").read_text(encoding="utf-8"))
if not isinstance(doctor, dict):
    raise SystemExit("doctor.json must be a JSON object")
if doctor.get("status") != "healthy":
    raise SystemExit("doctor status must be healthy")
if doctor.get("cyrune_home") != str(home_root):
    raise SystemExit("doctor cyrune_home must match workspace-local home")

required_response_fields = [
    "response_to",
    "correlation_id",
    "run_id",
    "evidence_id",
    "citation_bundle_id",
    "working_hash_after",
    "policy_pack_id",
]
for name in ["no-llm-response.json", "adapter-response.json"]:
    payload = json.loads((artifact_root / name).read_text(encoding="utf-8"))
    if not isinstance(payload, dict):
        raise SystemExit(f"{name} must be a JSON object")
    for key in required_response_fields:
        value = payload.get(key)
        if not isinstance(value, str) or value == "":
            raise SystemExit(f"{name} missing required non-empty field: {key}")

no_llm_follow = (artifact_root / "no-llm-evidence-follow.txt").read_text(encoding="utf-8")
if "accepted" not in no_llm_follow:
    raise SystemExit("no-llm evidence follow must contain accepted")

working = json.loads((artifact_root / "working-follow.txt").read_text(encoding="utf-8"))
if not isinstance(working, dict):
    raise SystemExit("working-follow must be a JSON object")
if "limit" not in working or "slots" not in working:
    raise SystemExit("working-follow must contain limit and slots")

policy = json.loads((artifact_root / "policy.json").read_text(encoding="utf-8"))
if not isinstance(policy, dict):
    raise SystemExit("policy.json must be a JSON object")
if "policy_pack" not in policy:
    raise SystemExit("policy.json must contain policy_pack")

adapter_follow = (artifact_root / "adapter-evidence-follow.txt").read_text(encoding="utf-8")
if "accepted" not in adapter_follow:
    raise SystemExit("adapter evidence follow must contain accepted")

launch_fail_closed = (artifact_root / "launch-fail-closed.txt").read_text(encoding="utf-8").strip()
if launch_fail_closed != "15":
    raise SystemExit("launch fail-closed must record exit code 15")

if not workspace_dir.is_dir():
    raise SystemExit("workspace directory must exist")
PY
