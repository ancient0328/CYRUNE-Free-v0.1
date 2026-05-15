#!/bin/sh
set -eu

dry_run=0
case "$#" in
  0)
    ;;
  1)
    if [ "$1" = "--dry-run" ]; then
      dry_run=1
    else
      exit 10
    fi
    ;;
  *)
    exit 10
    ;;
esac

if ! script_dir=$(CDPATH= cd -- "$(dirname -- "$0")" 2>/dev/null && pwd); then
  exit 11
fi
if ! workspace_root=$(CDPATH= cd -- "$script_dir/.." 2>/dev/null && pwd); then
  exit 11
fi

bootstrap_script="$script_dir/dev_front_bootstrap.sh"
set +e
"$bootstrap_script" >/dev/null 2>/dev/null
status=$?
set -e
if [ "$status" -ne 0 ]; then
  exit "$status"
fi

env_file="$workspace_root/target/developer-demo-front/env.sh"
if [ ! -f "$env_file" ]; then
  exit 14
fi

if ! . "$env_file"; then
  exit 14
fi

set +e
"$CYRUNE_DEV_FRONT_ROOT/bin/cyr" doctor >/dev/null
status=$?
set -e
if [ "$status" -ne 0 ]; then
  exit 16
fi

if [ -n "${CYRUNE_DEV_FRONT_WEZTERM_BIN:-}" ]; then
  wezterm_bin="$CYRUNE_DEV_FRONT_WEZTERM_BIN"
else
  wezterm_bin="$(command -v wezterm 2>/dev/null || true)"
fi

if [ -z "$wezterm_bin" ] || [ ! -x "$wezterm_bin" ]; then
  exit 15
fi

config_file="$CYRUNE_HOME/terminal/config/wezterm.lua"
launch_command="\"$wezterm_bin\" start --config-file \"$config_file\""

if [ "$dry_run" -eq 1 ]; then
  printf '%s\n' "$launch_command"
  exit 0
fi

exec "$wezterm_bin" start --config-file "$config_file"
