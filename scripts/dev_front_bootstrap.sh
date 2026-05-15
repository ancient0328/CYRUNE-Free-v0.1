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

manifest_path="$workspace_root/Cargo.toml"
build_root="$workspace_root/target/debug"
dev_front_root="$workspace_root/target/developer-demo-front"
bin_dir="$dev_front_root/bin"
home_dir="$dev_front_root/home"
env_file="$dev_front_root/env.sh"
runtime_bin="$build_root/cyrune-runtime-cli"
daemon_bin="$build_root/cyrune-daemon"
wrapper_cyr="$bin_dir/cyr"
wrapper_daemon="$bin_dir/cyrune-daemon"

if [ ! -f "$manifest_path" ]; then
  exit 11
fi

if ! cargo build --manifest-path "$manifest_path" -p cyrune-runtime-cli -p cyrune-daemon; then
  exit 12
fi

if [ ! -x "$runtime_bin" ] || [ ! -x "$daemon_bin" ]; then
  exit 13
fi

if ! mkdir -p "$bin_dir" "$home_dir"; then
  exit 13
fi

if ! cat >"$wrapper_cyr" <<EOF
#!/bin/sh
exec "$runtime_bin" "\$@"
EOF
then
  exit 13
fi

if ! cat >"$wrapper_daemon" <<EOF
#!/bin/sh
exec "$daemon_bin" "\$@"
EOF
then
  exit 13
fi

if ! chmod +x "$wrapper_cyr" "$wrapper_daemon"; then
  exit 13
fi

if ! cat >"$env_file" <<EOF
export CYRUNE_DEV_FRONT_ROOT="$dev_front_root"
export CYRUNE_HOME="$home_dir"
export CYRUNE_DAEMON_BIN="$wrapper_daemon"
export PATH="$bin_dir:\$PATH"
EOF
then
  exit 14
fi

printf '%s\n' \
  "CYRUNE_DEV_FRONT_ROOT=$dev_front_root" \
  "CYRUNE_HOME=$home_dir" \
  "ENV_FILE=$env_file"
