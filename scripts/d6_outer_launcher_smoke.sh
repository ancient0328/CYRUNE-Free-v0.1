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
artifact_root="$workspace_root/target/terminal-front-expansion/proof/D6"
accepted_root="$artifact_root/accepted"
fail_closed_root="$artifact_root/fail-closed"
validation_root="$artifact_root/validation"

release_manifest_file="$accepted_root/release-manifest.json"
doctor_health_file="$accepted_root/doctor-health.json"
launch_driver_file="$accepted_root/launch-driver.json"
launch_probe_file="$accepted_root/launch-probe.txt"
preflight_fail_file="$fail_closed_root/preflight-invalid-override.json"
preflight_fail_exit_file="$fail_closed_root/preflight-invalid-override-exit.txt"
launcher_fail_file="$fail_closed_root/launcher-missing-terminal.json"
launcher_fail_exit_file="$fail_closed_root/launcher-missing-terminal-exit.txt"
run_path_unresolved_file="$fail_closed_root/run-path-unresolved.json"

if ! command -v python3 >/dev/null 2>&1; then
  exit 16
fi

if [ ! -d "$shipping_root" ] || [ ! -f "$shipping_root/RELEASE_MANIFEST.json" ]; then
  python3 "$stage_script" >/dev/null
fi

if [ ! -d "$shipping_root" ]; then
  exit 12
fi
if [ ! -f "$shipping_root/RELEASE_MANIFEST.json" ]; then
  exit 12
fi

if ! rm -rf "$artifact_root"; then
  exit 12
fi
if ! mkdir -p "$accepted_root" "$fail_closed_root" "$validation_root"; then
  exit 12
fi

if ! cargo build --quiet --manifest-path "$workspace_root/Cargo.toml" --bin d6-proof-driver >/dev/null 2>/dev/null; then
  exit 17
fi

proof_driver_bin="$workspace_root/target/debug/d6-proof-driver"
if [ ! -x "$proof_driver_bin" ]; then
  exit 17
fi

cp "$shipping_root/RELEASE_MANIFEST.json" "$release_manifest_file"

bundle_root_rel="$(
  python3 - "$release_manifest_file" <<'PY'
import json
import sys

with open(sys.argv[1], encoding="utf-8") as handle:
    payload = json.load(handle)

value = payload.get("bundle_root_path")
if not isinstance(value, str) or value == "":
    raise SystemExit(1)

print(value)
PY
)"

dist_ok="$artifact_root/distribution-ok"
dist_missing_binding="$artifact_root/distribution-missing-binding"
home_ok="$artifact_root/home-ok"
home_run_path="$artifact_root/home-run-path"
workspace_ok="$artifact_root/workspace-ok"
cyr_ok="$dist_ok/bin/cyr"

cp -R "$shipping_root" "$dist_ok"
cp -R "$shipping_root" "$dist_missing_binding"
rm -f "$dist_missing_binding/$bundle_root_rel/adapter/bindings/cyrune-free-default.v0.1.json"

mkdir -p "$home_ok" "$home_run_path" "$workspace_ok"

fake_terminal="$artifact_root/fake-wezterm.sh"
cat >"$fake_terminal" <<EOF
#!/bin/sh
printf 'CYRUNE_HOME=%s\n' "\$CYRUNE_HOME" > "$launch_probe_file"
printf 'CYRUNE_DISTRIBUTION_ROOT=%s\n' "\$CYRUNE_DISTRIBUTION_ROOT" >> "$launch_probe_file"
printf 'ARGS=%s\n' "\$*" >> "$launch_probe_file"
exit 0
EOF
chmod 755 "$fake_terminal"

missing_terminal="$artifact_root/private/not-for-public/wezterm-missing"

CYRUNE_DISTRIBUTION_ROOT="$dist_ok" \
CYRUNE_HOME="$home_ok" \
CRANE_ROOT="/nonexistent/should-not-be-used" \
  "$cyr_ok" doctor >"$doctor_health_file"

"$proof_driver_bin" launch \
  --terminal-binary "$fake_terminal" \
  --cyrune-home "$home_ok" \
  --distribution-root "$dist_ok" >"$launch_driver_file"

set +e
"$proof_driver_bin" launch \
  --terminal-binary "$fake_terminal" \
  --cyrune-home "$home_ok" \
  --distribution-root relative/path >"$preflight_fail_file"
status=$?
set -e
printf 'exit=%s\n' "$status" >"$preflight_fail_exit_file"

set +e
"$proof_driver_bin" launch \
  --terminal-binary "$missing_terminal" \
  --cyrune-home "$home_ok" \
  --distribution-root "$dist_ok" >"$launcher_fail_file"
status=$?
set -e
printf 'exit=%s\n' "$status" >"$launcher_fail_exit_file"

CYRUNE_DISTRIBUTION_ROOT="$dist_missing_binding" \
CYRUNE_HOME="$home_run_path" \
CRANE_ROOT="/nonexistent/should-not-be-used" \
  "$dist_missing_binding/bin/cyr" run --no-llm --input "d6 run-path unresolved proof" >"$run_path_unresolved_file"

python3 - \
  "$accepted_root" \
  "$fail_closed_root" \
  "$dist_ok" \
  "$bundle_root_rel" \
  "$home_ok" \
  "$missing_terminal" \
  "$launch_probe_file" \
  "$fake_terminal" <<'PY'
import json
import pathlib
import re
import sys

accepted_root = pathlib.Path(sys.argv[1])
fail_closed_root = pathlib.Path(sys.argv[2])
dist_ok = pathlib.Path(sys.argv[3])
bundle_root_rel = pathlib.Path(sys.argv[4])
home_ok = pathlib.Path(sys.argv[5])
missing_terminal = pathlib.Path(sys.argv[6])
launch_probe_file = pathlib.Path(sys.argv[7])
fake_terminal = pathlib.Path(sys.argv[8])
bundle_root = dist_ok / bundle_root_rel

required_files = [
    accepted_root / "release-manifest.json",
    accepted_root / "doctor-health.json",
    accepted_root / "launch-driver.json",
    fail_closed_root / "preflight-invalid-override.json",
    fail_closed_root / "preflight-invalid-override-exit.txt",
    fail_closed_root / "launcher-missing-terminal.json",
    fail_closed_root / "launcher-missing-terminal-exit.txt",
    fail_closed_root / "run-path-unresolved.json",
]
for path in required_files:
    if not path.is_file():
        raise SystemExit(f"missing artifact: {path.name}")
if not launch_probe_file.is_file():
    raise SystemExit("missing artifact: launch-probe.txt")

manifest = json.loads((accepted_root / "release-manifest.json").read_text(encoding="utf-8"))
if manifest.get("bundle_root_path") != "share/cyrune/bundle-root":
    raise SystemExit("release-manifest bundle_root_path mismatch")
if manifest.get("home_template_path") != "share/cyrune/home-template":
    raise SystemExit("release-manifest home_template_path mismatch")

doctor = json.loads((accepted_root / "doctor-health.json").read_text(encoding="utf-8"))
if doctor.get("status") != "healthy":
    raise SystemExit("doctor health must be healthy")
if doctor.get("distribution_root") != str(dist_ok):
    raise SystemExit("doctor distribution_root mismatch")
if doctor.get("bundle_root") != str(bundle_root):
    raise SystemExit("doctor bundle_root mismatch")

launch_payload = json.loads((accepted_root / "launch-driver.json").read_text(encoding="utf-8"))
if launch_payload.get("status") != "launched":
    raise SystemExit("launch-driver must be launched")
if launch_payload.get("exit_code") != 0:
    raise SystemExit("launch-driver exit_code must be 0")
invocation = launch_payload.get("invocation")
if not isinstance(invocation, dict):
    raise SystemExit("launch-driver invocation must be an object")
if invocation.get("program") != str(fake_terminal):
    raise SystemExit("launch-driver program mismatch")
args = invocation.get("args")
if args != ["start", "--config-file", str(home_ok / "terminal" / "config" / "wezterm.lua")]:
    raise SystemExit("launch-driver args mismatch")
env_payload = invocation.get("env")
if not isinstance(env_payload, dict):
    raise SystemExit("launch-driver env must be an object")
if env_payload.get("CYRUNE_HOME") != str(home_ok):
    raise SystemExit("launch-driver env missing CYRUNE_HOME")
if env_payload.get("CYRUNE_DISTRIBUTION_ROOT") != str(dist_ok):
    raise SystemExit("launch-driver env missing CYRUNE_DISTRIBUTION_ROOT")
if "BUNDLE_ROOT" in env_payload:
    raise SystemExit("launch-driver env must not expose BUNDLE_ROOT")

probe = launch_probe_file.read_text(encoding="utf-8")
if f"CYRUNE_HOME={home_ok}" not in probe:
    raise SystemExit("launch probe missing CYRUNE_HOME")
if f"CYRUNE_DISTRIBUTION_ROOT={dist_ok}" not in probe:
    raise SystemExit("launch probe missing CYRUNE_DISTRIBUTION_ROOT")
if "ARGS=start --config-file" not in probe:
    raise SystemExit("launch probe missing args")

preflight_payload = json.loads((fail_closed_root / "preflight-invalid-override.json").read_text(encoding="utf-8"))
if preflight_payload.get("status") != "failed":
    raise SystemExit("preflight-invalid-override must fail")
if preflight_payload.get("surface") != "preflight_failure":
    raise SystemExit("preflight-invalid-override surface mismatch")
if preflight_payload.get("reason") != "invalid_distribution_root_override":
    raise SystemExit("preflight-invalid-override reason mismatch")
if preflight_payload.get("message") != "packaged distribution root override is invalid":
    raise SystemExit("preflight-invalid-override message mismatch")

launcher_payload = json.loads((fail_closed_root / "launcher-missing-terminal.json").read_text(encoding="utf-8"))
if launcher_payload.get("status") != "failed":
    raise SystemExit("launcher-missing-terminal must fail")
if launcher_payload.get("surface") != "launcher_failure":
    raise SystemExit("launcher-missing-terminal surface mismatch")
if launcher_payload.get("reason") != "terminal_binary_unavailable":
    raise SystemExit("launcher-missing-terminal reason mismatch")
if launcher_payload.get("message") != "launcher terminal binary is unavailable":
    raise SystemExit("launcher-missing-terminal message mismatch")

run_path_payload = json.loads((fail_closed_root / "run-path-unresolved.json").read_text(encoding="utf-8"))
if run_path_payload.get("reason_kind") != "binding_unresolved":
    raise SystemExit("run-path-unresolved reason_kind mismatch")
rule_id = run_path_payload.get("rule_id")
if not isinstance(rule_id, str) or not rule_id.startswith("BND-"):
    raise SystemExit("run-path-unresolved rule_id mismatch")

for name in [
    "preflight-invalid-override-exit.txt",
    "launcher-missing-terminal-exit.txt",
]:
    text = (fail_closed_root / name).read_text(encoding="utf-8").strip()
    if not text.startswith("exit="):
        raise SystemExit(f"{name} must start with exit=")
    exit_code = int(text.split("=", 1)[1])
    if exit_code == 0:
        raise SystemExit(f"{name} must be non-zero")

raw_leak_candidates = [
    str(dist_ok),
    str(bundle_root),
    str(missing_terminal),
    "No such file",
    "no such file",
]
for path in [
    fail_closed_root / "preflight-invalid-override.json",
    fail_closed_root / "launcher-missing-terminal.json",
    fail_closed_root / "run-path-unresolved.json",
]:
    text = path.read_text(encoding="utf-8")
    for candidate in raw_leak_candidates:
        if candidate and candidate in text:
            raise SystemExit(f"{path.name} leaked raw detail: {candidate}")
    if re.search(r'"/[^"]+"', text):
        raise SystemExit(f"{path.name} leaked an absolute path literal")
PY
